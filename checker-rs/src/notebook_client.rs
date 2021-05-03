use crate::NotebookUser;

use std::io;

use enochecker::{
    tokio::{
        io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufStream},
        net::TcpStream,
    },
    CheckerError, CheckerRequest, CheckerResult,
};

use tracing::{debug, error, info, instrument, warn};
/// reads until the delimiter given in delim is found or EOF if found or another Error occurs
///
/// # Return
/// Number of bytes read
pub async fn read_until_slice<'a, T: AsyncBufRead + Unpin>(
    stream: &'a mut T,
    delim: &'a [u8],
    buf: &'a mut Vec<u8>,
) -> io::Result<usize> {
    let mut bytes_read: usize = 0;
    let mut inc_read;

    let delim_end = *delim.last().expect("DELIMITER must not be empty");

    loop {
        inc_read = stream.read_until(delim_end, buf).await?;
        if inc_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Delimiter was not found",
            ));
        }
        bytes_read += inc_read;
        if bytes_read >= delim.len() && &buf[(buf.len() - delim.len())..buf.len()] == delim {
            return Ok(bytes_read);
        }
    }
}

enum CheckerErrorUnfilled<T: std::fmt::Debug + 'static> {
    Mumble,
    Offline,
    InternalError(T),
}

impl From<io::Error> for CheckerErrorUnfilled<io::Error> {
    fn from(e: std::io::Error) -> CheckerErrorUnfilled<std::io::Error> {
        match e.kind() {
            io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset => CheckerErrorUnfilled::Offline,
            io::ErrorKind::UnexpectedEof => CheckerErrorUnfilled::Mumble,
            _ => CheckerErrorUnfilled::InternalError(e),
        }
    }
}

impl<T: std::fmt::Debug + 'static> CheckerErrorUnfilled<T> {
    fn with_message(self, err_msg: &'static str) -> CheckerError {
        match self {
            CheckerErrorUnfilled::Offline => CheckerError::Offline(err_msg),
            CheckerErrorUnfilled::Mumble => CheckerError::Mumble(err_msg),
            CheckerErrorUnfilled::InternalError(err) => {
                error!("Internal error: {:?}", err);
                CheckerError::InternalError("Internal Error")
            }
        }
    }
}

pub fn bytes_debug_repr(bytes: &[u8]) -> String {
    let mut bytes_repr = "b'".to_owned();
    for b in bytes {
        let b_escaped: Vec<_> = std::ascii::escape_default(*b).collect();
        bytes_repr.push_str(std::str::from_utf8(&b_escaped).unwrap());
    }
    bytes_repr.push('\'');
    bytes_repr
}

#[derive(Debug)]
pub struct NotebookClient {
    pub conn: BufStream<TcpStream>,
    pub user: Option<NotebookUser>,
}

const PROMPT: &[u8] = b"\n> ";

impl NotebookClient {
    const SERVICE_PORT: u16 = 2323;

    #[instrument("CONNECT")]
    pub async fn connect(request: &CheckerRequest) -> CheckerResult<Self> {
        info!("Connecting to service");
        let mut conn =
            match TcpStream::connect(format!("{}:{}", request.address, Self::SERVICE_PORT)).await {
                Ok(conn) => BufStream::new(conn),
                Err(e) => {
                    info!("Failed to connect to service!, {:?}", e);
                    return Err(CheckerError::Offline("Connection to service failed"));
                }
            };

        debug!("Fetching Welcome Banner");
        let mut welcome_banner = Vec::with_capacity(256);
        let bytes_read = match read_until_slice(&mut conn, PROMPT, &mut welcome_banner).await {
            Err(_) | Ok(0) => Err(CheckerError::Mumble("Failed to fetch welcome banner")),
            Ok(r) => Ok(r),
        }?;

        if &welcome_banner[(bytes_read - 3)..bytes_read] != b"\n> " {
            warn!(
                "Welcome Banner fetching failed response {}",
                bytes_debug_repr(&welcome_banner)
            );
            return Err(CheckerError::Mumble("Failed to fetch Welcome Banner"));
        }

        Ok(Self { conn, user: None })
    }

    #[instrument("REGISTER")]
    pub async fn register(&mut self, user: &NotebookUser) -> CheckerResult<()> {
        info!("Registering User: {:?}", user);

        let login_str = format!("reg {} {}\n", user.username, user.password);
        self.conn
            .write_all(login_str.as_bytes())
            .await
            .map_err(|_| {
                warn!("User registration failed {:?}", user);
                CheckerError::Mumble("Failed to register user")
            })?;
        self.conn.flush().await.map_err(|e| {
            info!("Flush failed -- {}", e);
            CheckerError::Mumble("Falied to register user")
        })?;

        debug!("Waiting for registration to complete");
        let mut response = Vec::with_capacity(100);
        match read_until_slice(&mut self.conn, PROMPT, &mut response).await {
            Err(e) => {
                info!("Socket error waiting for user registration {:?}", e);
                Err(CheckerError::Mumble("Registration failed"))
            }
            Ok(0) => {
                info!("Unexpected EOF waiting for user registration");
                Err(CheckerError::Mumble("Registration failed"))
            }
            Ok(r) => Ok(r),
        }?;

        if response != b"User successfully registered\n> " {
            info!("Unexpected response: {}", bytes_debug_repr(&response));
            return Err(CheckerError::Mumble("Registration Failed"));
        }

        Ok(())
    }

    #[instrument("LOGIN")]
    pub async fn login(&mut self, user: NotebookUser) -> CheckerResult<()> {
        info!("Logging in as {:?}", &user);
        self.user = Some(user);
        let user_ref = self.user.as_ref().unwrap();
        let conn = &mut self.conn;

        let mut response_buf = Vec::with_capacity(64);
        async {
            let login_str = format!("log {} {}\n", user_ref.username, user_ref.password);
            conn.write_all(login_str.as_bytes()).await?;
            conn.flush().await?;

            debug!("Waiting for login");
            read_until_slice(conn, PROMPT, &mut response_buf).await
        }
        .await
        .map_err(|e: std::io::Error| {
            warn!("Connection-Error on Login: {}", e);
            CheckerErrorUnfilled::from(e).with_message("Login failed")
        })?;

        if response_buf != b"Successfully logged in!\n> " {
            info!("Unexpected response: {}", bytes_debug_repr(&response_buf));
            return Err(CheckerError::Mumble("Login failed"));
        }

        Ok(())
    }

    #[instrument("SET_NOTE")]
    pub async fn set_note(&mut self, note: &str) -> CheckerResult<()> {
        info!("Deploying Note {}", note);

        let conn = &mut self.conn;

        let mut response_buf = Vec::with_capacity(64);
        async {
            let request = format!("set {}\n", note);
            debug!("running: '{}'", request);
            conn.write_all(request.as_bytes()).await?;
            conn.flush().await?;

            read_until_slice(conn, PROMPT, &mut response_buf).await
        }
        .await
        .map_err(|e| {
            warn!("Connection-Error: {}", e);
            CheckerErrorUnfilled::from(e).with_message("Set-Note connection error")
        })?;

        let response = String::from_utf8(response_buf)?;
        let note_id = response
            .split("Note saved! ID is ")
            .nth(1)
            .and_then(|substr| substr.split('!').next())
            .ok_or(CheckerError::Mumble("Failed to set note"))?;

        self.user.as_mut().unwrap().note = Some(note.to_string());
        self.user.as_mut().unwrap().note_id = Some(note_id.to_string());

        Ok(())
    }

    #[instrument("GET_NOTE")]
    pub async fn get_note(&mut self) -> CheckerResult<String> {
        info!(
            "Retirieving Note for user {:?}",
            self.user.as_ref().expect("Getting note of NONE-User")
        );

        let conn = &mut self.conn;
        let note_id = self
            .user
            .as_ref()
            .unwrap()
            .note_id
            .as_ref()
            .expect("No NoteId for saved user");

        let mut response_buf = Vec::with_capacity(256);
        async {
            let request = format!("get {}\n", note_id);
            debug!("request: '{}'", request);
            conn.write_all(request.as_bytes()).await?;
            conn.flush().await?;

            read_until_slice(conn, PROMPT, &mut response_buf).await
        }
        .await
        .map_err(|e| {
            info!("Socketerror when retrieving note: {:?}", e);
            CheckerErrorUnfilled::from(e).with_message("Connection error upon fetching note")
        })?;

        response_buf.truncate(response_buf.len() - 3);
        Ok(String::from_utf8(response_buf)?)
    }

    #[instrument("GET_HELP")]
    pub async fn get_help(&mut self) -> CheckerResult<String> {
        info!("Getting help");

        let conn = &mut self.conn;
        let mut response_buf = Vec::with_capacity(512);
        async {
            conn.write_all(b"help\n").await?;
            conn.flush().await?;

            read_until_slice(conn, PROMPT, &mut response_buf).await
        }
        .await
        .map_err(|e| {
            warn!("Socketerror upon getting help text {:?}", e);
            CheckerErrorUnfilled::from(e).with_message("Connection error upon getting help")
        })?;
        response_buf.truncate(response_buf.len() - 3);
        Ok(String::from_utf8(response_buf)?)
    }

    #[instrument("GET_USERS")]
    pub async fn get_users(&mut self) -> CheckerResult<Vec<String>> {
        info!("Listing users");

        let conn = &mut self.conn;
        let mut response_buf = Vec::with_capacity(512);
        async {
            conn.write_all(b"user\n").await?;
            conn.flush().await?;

            read_until_slice(conn, PROMPT, &mut response_buf).await
        }
        .await
        .map_err(|e| {
            warn!("Socketerror upon getting help text {:?}", e);
            CheckerErrorUnfilled::from(e).with_message("Connection error upon getting help")
        })?;

        response_buf.truncate(response_buf.len() - 3);
        let users = String::from_utf8(response_buf)?;

        let user_arr: Vec<String> = users
            .split('\n')
            .filter_map(|line| {
                line.split(": ")
                    .nth(1)
                    .map(std::string::ToString::to_string)
            })
            .collect();

        Ok(user_arr)
    }

    #[instrument("GET_NOTES")]
    pub async fn get_notes(&mut self) -> CheckerResult<Vec<String>> {
        info!("Getting Note List");

        let conn = &mut self.conn;
        let mut response_buf = Vec::with_capacity(512);
        async {
            conn.write_all(b"list\n").await?;
            conn.flush().await?;

            read_until_slice(conn, PROMPT, &mut response_buf).await
        }
        .await
        .map_err(|e| {
            warn!("Socketerror upon getting note listing {:?}", e);
            CheckerErrorUnfilled::from(e).with_message("Connection error upon getting note listing")
        })?;

        response_buf.truncate(response_buf.len() - 3);
        let notes = String::from_utf8(response_buf)?;

        let note_arr: Vec<String> = notes
            .split('\n')
            .filter_map(|line| {
                line.split(": ")
                    .nth(1)
                    .map(std::string::ToString::to_string)
            })
            .collect();

        Ok(note_arr)
    }
}
