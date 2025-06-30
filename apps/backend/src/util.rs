use std::{
    convert::Infallible,
    io::{self, Write as _},
    net::{IpAddr, SocketAddr},
};

use axum::{
    Json,
    extract::{ConnectInfo, Request, ws::CloseFrame},
    http::{Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{
    Local,
    format::{DelayedFormat, StrftimeItems},
};
use rand::seq::{IndexedRandom as _, IteratorRandom as _};
use serde::Serialize;

use self::direction_arrow::DirectionArrow;

mod direction_arrow;

macro_rules! log_error {
    ($identifier:literal, $error:expr) => {
        eprintln!(concat!("Error(", $identifier, "): {:#}"), $error);
    };
}

pub(crate) use log_error;

/// This function unwraps the Result which is Infallible,
/// indicating unwrapping this Result will never fail.
#[inline(always)]
pub fn unwrap_infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => match err {},
    }
}

#[inline(always)]
pub fn get_time_string() -> DelayedFormat<StrftimeItems<'static>> {
    Local::now().format("%F %T")
}

#[inline(always)]
pub fn generate_name() -> String {
    let mut thread_rng = rand::rng();
    let three_numbers = (b'A'..=b'Z').choose_multiple(&mut thread_rng, 4);
    let mut pseudo_id = Vec::new();
    pseudo_id.resize_with(9, || {
        three_numbers.choose(&mut thread_rng).copied().unwrap()
    });
    unsafe { String::from_utf8_unchecked(pseudo_id) }
}

pub struct JsonResponse<T>
where
    T: Serialize,
{
    pub status_code: StatusCode,
    pub content: T,
}

impl<T> IntoResponse for JsonResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (self.status_code, Json(self.content)).into_response()
    }
}

pub async fn log_http_middleware(
    uri: Uri,
    method: Method,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let time = get_time_string();
    let ip = addr.ip().to_string();
    let url = match uri.query() {
        Some(query) => format!("{}?{}", uri.path(), query),
        None => uri.path().to_owned(),
    };
    let res = next.run(req).await;
    let status = res.status();
    let _: Result<(), io::Error> = (|| {
        let mut stdout = io::stdout().lock();
        write!(stdout, "[HTTP] ")?;
        if status.as_u16() >= 400 {
            write!(stdout, "\x1b[31m")?;
        } else if status.as_u16() >= 300 {
            write!(stdout, "\x1b[34m")?;
        }
        write!(
            stdout,
            "[{}] {} {} {} {} ({})",
            time,
            ip,
            DirectionArrow::from(&status),
            method,
            url,
            status.as_str()
        )?;
        if status.as_u16() >= 300 {
            write!(stdout, "\x1b[0m")?;
        }
        writeln!(stdout)?;
        Ok(())
    })();
    res
}

pub enum WebSocketReceiveAction<'a> {
    GotPong,
    GotText(&'a str),
    GotClose(Option<CloseFrame>),
}

pub enum WebSocketSendAction<'a> {
    SendPing,
    SendText(&'a str),
    SendClose(CloseFrame),
}

pub enum WebSocketAction<'a> {
    Connect,
    Disconnect,
    Receive(WebSocketReceiveAction<'a>),
    Send(Result<WebSocketSendAction<'a>, WebSocketSendAction<'a>>),
}

impl<'a> From<WebSocketReceiveAction<'a>> for WebSocketAction<'a> {
    fn from(value: WebSocketReceiveAction<'a>) -> Self {
        Self::Receive(value)
    }
}

impl<'a> From<Result<WebSocketSendAction<'a>, WebSocketSendAction<'a>>> for WebSocketAction<'a> {
    fn from(value: Result<WebSocketSendAction<'a>, WebSocketSendAction<'a>>) -> Self {
        Self::Send(value)
    }
}

pub fn log_ws<'a, T>(ip: IpAddr, action: T)
where
    T: Into<WebSocketAction<'a>>,
{
    let action = action.into();
    let time = get_time_string();
    let _: Result<(), io::Error> = (|| {
        let mut stdout = io::stdout().lock();
        write!(stdout, "[ WS ] ")?;
        if matches!(action, WebSocketAction::Send(Err(_))) {
            write!(stdout, "\x1b[31m")?;
        }
        write!(
            stdout,
            "[{}] {} {} ",
            time,
            ip,
            DirectionArrow::from(&action)
        )?;
        match action {
            WebSocketAction::Connect => write!(stdout, "Connected")?,
            WebSocketAction::Disconnect => write!(stdout, "Disconnected")?,
            WebSocketAction::Receive(ref inner_action) => match inner_action {
                WebSocketReceiveAction::GotPong => write!(stdout, "Sent ping")?,
                WebSocketReceiveAction::GotText(text) => {
                    write!(stdout, "Sent text data: {}", text)?
                }
                WebSocketReceiveAction::GotClose(c) => {
                    if let Some(cf) = c {
                        write!(stdout, "Sent close: {} \"{}\"", cf.code, cf.reason)?
                    } else {
                        write!(stdout, "Sent close without close frame")?
                    }
                }
            },
            WebSocketAction::Send(ref inner_action) => match inner_action {
                Ok(WebSocketSendAction::SendPing) => write!(stdout, "Sent ping")?,
                Ok(WebSocketSendAction::SendText(text)) => {
                    write!(stdout, "Sent text data: {}", text)?
                }
                Ok(WebSocketSendAction::SendClose(cf)) => {
                    write!(stdout, "Sent close: {} \"{}\"", cf.code, cf.reason)?
                }
                Err(WebSocketSendAction::SendPing) => write!(stdout, "Could not send ping")?,
                Err(WebSocketSendAction::SendText(text)) => {
                    write!(stdout, "Could not send text data: {}", text)?
                }
                Err(WebSocketSendAction::SendClose(cf)) => write!(
                    stdout,
                    "Could not send close: {} \"{}\"",
                    cf.code, cf.reason
                )?,
            },
        };
        if matches!(action, WebSocketAction::Send(Err(_))) {
            write!(stdout, "\x1b[0m")?;
        }
        writeln!(stdout)?;
        Ok(())
    })();
}

#[cfg(windows)]
#[inline(always)]
pub fn windows_setup() {
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::{
        Foundation::{GetLastError, HANDLE},
        System::{
            Console::{ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, SetConsoleMode},
            Threading::ExitProcess,
        },
    };
    #[inline(always)]
    unsafe fn enable_vt(handle: HANDLE) {
        unsafe {
            let mut console_mode = 0;
            if GetConsoleMode(handle, &mut console_mode) == 0 {
                eprintln!("Error: failed to get console mode");
                ExitProcess(GetLastError());
            }
            console_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
            if SetConsoleMode(handle, console_mode) == 0 {
                eprintln!("Error: failed to set console mode");
                ExitProcess(GetLastError());
            }
        }
    }
    unsafe {
        let std_output: HANDLE = io::stdout().as_raw_handle();
        let std_error: HANDLE = io::stderr().as_raw_handle();
        enable_vt(std_output);
        if std_output != std_error {
            enable_vt(std_error);
        }
    }
}
