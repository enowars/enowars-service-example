use crate::NotebookUser;

use std::io;

use enochecker::{
    result::{CheckerError, CheckerResult, CheckerfromIOResult, IntoCheckerResult},
    tokio::{
        io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufStream},
        net::TcpStream,
    },
    CheckerRequest,
};

use tracing::{debug, info, instrument, warn};
/// reads until the delimiter given in delim is found or EOF if found or another Error occurs
///
/// # Return
/// Number of bytes read

const PROMPT: &[u8] = b"\n> ";
const WELCOME_BANNER: &[u8] = b"Welcome to the 1337 n0t3b00k!\n> ";
const REGISTRATION_SUCCESS: &[u8] = b"User successfully registered";
const LOGIN_SUCCESS: &[u8] = b"Successfully logged in!";

pub async fn read_until_slice<'a, T: AsyncBufRead + Unpin>(
    stream: &'a mut T,
    delim: &'a [u8],
    buf: &'a mut Vec<u8>,
) -> io::Result<usize> {
    let mut bytes_read: usize = 0;
    let mut inc_read;

    let delim_end = *delim.last().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "read_until_slice recieved empty delimiter",
        )
    })?;

    loop {
        inc_read = stream.read_until(delim_end, buf).await?;
        if inc_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Delimiter could not be found before EOF was hit",
            ));
        }
        bytes_read += inc_read;
        if buf.ends_with(delim) {
            buf.truncate(buf.len() - delim.len());
            return Ok(bytes_read);
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

impl NotebookClient {
    const SERVICE_PORT: u16 = 2323;

    #[instrument("CONNECT")]
    pub async fn connect(request: &CheckerRequest) -> CheckerResult<Self> {
        info!("Connecting to service");
        let mut conn =
            match TcpStream::connect((request.address.as_str(), Self::SERVICE_PORT)).await {
                Ok(conn) => BufStream::new(conn),
                Err(e) => {
                    info!("Failed to connect to service!, {:?}", e);
                    return Err(CheckerError::Offline("Connection to service failed"));
                }
            };

        debug!("Fetching Welcome Banner");
        let mut welcome_banner = [0; WELCOME_BANNER.len()];
        conn.read_exact(&mut welcome_banner)
            .await
            .into_checker_result("Connection error on login")?;

        if &welcome_banner != WELCOME_BANNER {
            warn!(
                "Welcome Banner fetching failed, response {}",
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
        read_until_slice(&mut self.conn, PROMPT, &mut response)
            .await
            .into_checker_result("Registration closed connection")?;

        if response != REGISTRATION_SUCCESS {
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
        .into_checker_result("Login failed")?;

        if response_buf != LOGIN_SUCCESS {
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
        .into_checker_result("Set-Note connection error")?;

        let response = String::from_utf8(response_buf).into_mumble("Response is invalid UTF8")?;
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
        .into_checker_result("Connection error upon fetching note")?;

        Ok(String::from_utf8(response_buf).into_mumble("Response contains invalid UTF8")?)
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
        .into_checker_result("Connection error upon getting help")?;

        Ok(String::from_utf8(response_buf).into_mumble("Response contains invalid UTF8")?)
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
        .into_checker_result("Connection error trying to get user list")?;

        let users = String::from_utf8(response_buf).into_mumble("Response is invalid UTF8")?;

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
        .into_checker_result("Connection error upon getting note listing")?;

        let notes = String::from_utf8(response_buf).into_mumble("Response is invalid UTF8")?;

        let note_arr: Vec<String> = notes
            .split('\n')
            .filter_map(|line| line.split(": ").nth(1).map(ToString::to_string))
            .collect();

        Ok(note_arr)
    }
}
