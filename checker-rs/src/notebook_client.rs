use crate::NotebookUser;

use enochecker::{
    tokio::{
        io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufStream},
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
    error_message: &'static str,
) -> CheckerResult<usize> {
    let mut bytes_read: usize = 0;
    let mut inc_read;
    loop {
        inc_read = stream
            .read_until(
                *delim.last().ok_or_else(|| {
                    error!("Delimiter is empty!");
                    CheckerError::InternalError("Delimiter must not be empty")
                })?,
                buf,
            )
            .await
            .map_err(|e| {
                warn!("Read until failed: {:?}", e);
                CheckerError::Mumble(error_message)
            })?;
        if inc_read == 0 {
            return Ok(bytes_read);
        }
        bytes_read += inc_read;
        if bytes_read >= delim.len() {
            if &buf[(buf.len() - delim.len())..buf.len()] == delim {
                return Ok(bytes_read);
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
    conn: BufStream<TcpStream>,
    user: Option<NotebookUser>,
}

const PROMPT: &[u8] = b"\n> ";

impl NotebookClient {
    const SERVICE_PORT: u16 = 22323;

    #[instrument("CONNECT")]
    pub async fn connect(request: &CheckerRequest) -> CheckerResult<Self> {
        info!("Connecting to service");
        let mut conn =
            match TcpStream::connect(format!("{}:{}", request.address, Self::SERVICE_PORT)).await {
                Ok(conn) => BufStream::new(conn),
                Err(e) => {
                    info!("Failed to connect to service!");
                    return Err(CheckerError::Offline("Connection to service failed"));
                }
            };

        debug!("Fetching Welcome Banner");
        let mut welcome_banner = Vec::with_capacity(256);
        let bytes_read = match read_until_slice(
            &mut conn,
            PROMPT,
            &mut welcome_banner,
            "Socket closed unexpectedly",
        )
        .await
        {
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

        Ok(Self {
            conn: conn,
            user: None,
        })
    }

    #[instrument("REGISTER")]
    pub async fn register(&mut self, user: &NotebookUser) -> CheckerResult<()> {
        info!("Registering User: {:?}", user);

        let login_str = format!("reg {} {}\n", user.username, user.password);
        self.conn.write(login_str.as_bytes()).await.map_err(|_| {
            warn!("User registration failed {:?}", user);
            CheckerError::Mumble("Failed to register user")
        })?;
        self.conn.flush().await.map_err(|e| {
            info!("Flush failed -- {}", e);
            return CheckerError::Mumble("Falied to register user");
        })?;

        debug!("Waiting for registration to complete");
        let mut response = Vec::with_capacity(100);
        let foo = match read_until_slice(
            &mut self.conn,
            PROMPT,
            &mut response,
            "Socket closed unexpectedly",
        )
        .await
        {
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

    pub async fn login(&mut self, user: &NotebookUser) -> CheckerResult<()> {
        unimplemented!()
    }

    pub async fn set_note(&mut self, note: &str) -> CheckerResult<()> {
        unimplemented!()
    }

    pub async fn get_note(&mut self) -> CheckerResult<String> {
        unimplemented!()
    }

    pub async fn get_help(&mut self) -> CheckerResult<(String)> {
        unimplemented!()
    }

    pub async fn get_users(&mut self) -> CheckerResult<Vec<String>> {
        unimplemented!()
    }

    pub async fn get_notes(&mut self) -> CheckerResult<Vec<String>> {
        unimplemented!()
    }
}
