#![allow(dead_code)]

use std::{
    convert::Infallible,
    fmt,
    net::{IpAddr, SocketAddr},
};

use axum::{
    extract::{ws::CloseFrame, ConnectInfo, Request},
    http::{header, HeaderName, HeaderValue, Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::{
    format::{DelayedFormat, StrftimeItems},
    Local,
};
use rand::seq::{IteratorRandom as _, SliceRandom as _};

use self::direction_arrow::DirectionArrow;

mod direction_arrow;

pub mod deser_utils;

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
pub fn get_local_time() -> DelayedFormat<StrftimeItems<'static>> {
    Local::now().format("%F %T")
}

#[inline(always)]
pub fn generate_name() -> String {
    let mut thread_rng = rand::thread_rng();
    let three_numbers = (b'0'..=b'9').choose_multiple(&mut thread_rng, 3);
    let mut pseudo_id = Vec::new();
    pseudo_id.resize_with(9, || {
        three_numbers.choose(&mut thread_rng).copied().unwrap()
    });
    unsafe { String::from_utf8_unchecked(pseudo_id) }
}

pub struct SimpleResponse<CT>
where
    CT: IntoResponse,
{
    pub status_code: StatusCode,
    pub content_type: &'static str,
    pub content: CT,
}

impl<CT> IntoResponse for SimpleResponse<CT>
where
    CT: IntoResponse,
{
    fn into_response(self) -> Response {
        (
            self.status_code,
            [(header::CONTENT_TYPE, self.content_type)],
            self.content,
        )
            .into_response()
    }
}

pub struct SimpleResponseWithHeaders<CT, HV, const HN: usize>
where
    CT: IntoResponse,
    HV: TryInto<HeaderValue>,
    HV::Error: fmt::Display,
{
    pub original_response: SimpleResponse<CT>,
    pub headers: [(HeaderName, HV); HN],
}

impl<CT, HV, const HN: usize> IntoResponse for SimpleResponseWithHeaders<CT, HV, HN>
where
    CT: IntoResponse,
    HV: TryInto<HeaderValue>,
    HV::Error: fmt::Display,
{
    fn into_response(self) -> Response {
        (self.headers, self.original_response).into_response()
    }
}

pub async fn log_http_middleware(
    uri: Uri,
    method: Method,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let time = get_local_time();
    let ip = addr.ip().to_string();
    let url = match uri.query() {
        Some(query) => format!("{}?{}", uri.path(), query),
        None => uri.path().to_owned(),
    };
    let res = next.run(req).await;
    let status = res.status();
    print!("[HTTP] ");
    if status.as_u16() >= 400 {
        print!("\x1b[31m");
    } else if status.as_u16() >= 300 {
        print!("\x1b[34m");
    }
    print!(
        "[{}] {} {} {} {} ({})",
        time,
        ip,
        DirectionArrow::from(status),
        method,
        url,
        status.as_str()
    );
    if status.as_u16() >= 300 {
        print!("\x1b[0m");
    }
    println!();
    res
}

#[derive(Debug, Clone, Copy)]
pub enum WebSocketReceiveAction<'a> {
    GotPong,
    GotText(&'a str),
    GotBinary,
    GotClose(&'a Option<CloseFrame<'static>>),
}

#[derive(Debug, Clone, Copy)]
pub enum WebSocketSendAction<'a> {
    SendPing,
    SendText(&'a str),
    SendBinary,
    SendClose(&'a CloseFrame<'static>),
}

#[derive(Debug, Clone, Copy)]
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
    let time = get_local_time();
    print!("[ WS ] ");
    if matches!(action, WebSocketAction::Send(Err(_))) {
        print!("\x1b[31m");
    }
    print!("[{}] {} {} ", time, ip, DirectionArrow::from(action));
    match action {
        WebSocketAction::Connect => print!("Connected"),
        WebSocketAction::Disconnect => print!("Disconnected"),
        WebSocketAction::Receive(inner_action) => match inner_action {
            WebSocketReceiveAction::GotPong => print!("Sent ping"),
            WebSocketReceiveAction::GotText(text) => print!("Sent text data: {}", text),
            WebSocketReceiveAction::GotBinary => print!("Sent binary data: (Cannot process)"),
            WebSocketReceiveAction::GotClose(c) => {
                if let Some(cf) = c {
                    println!("Sent close: {} \"{}\"", cf.code, cf.reason);
                } else {
                    println!("Sent close without close frame");
                }
            }
        },
        WebSocketAction::Send(inner_action) => match inner_action {
            Ok(WebSocketSendAction::SendPing) => print!("Sent ping"),
            Ok(WebSocketSendAction::SendText(text)) => print!("Sent text data: {}", text),
            Ok(WebSocketSendAction::SendBinary) => print!("Sent binary data: (Cannot process)"),
            Ok(WebSocketSendAction::SendClose(cf)) => {
                print!("Sent close: {} \"{}\"", cf.code, cf.reason)
            }
            Err(WebSocketSendAction::SendPing) => print!("Could not send ping"),
            Err(WebSocketSendAction::SendText(text)) => {
                print!("Could not send text data: {}", text)
            }
            Err(WebSocketSendAction::SendBinary) => {
                print!("Could not send binary data: (Cannot process)")
            }
            Err(WebSocketSendAction::SendClose(cf)) => {
                print!("Could not send close: {} \"{}\"", cf.code, cf.reason)
            }
        },
    }
    if matches!(action, WebSocketAction::Send(Err(_))) {
        print!("\x1b[0m");
    }
    println!();
}

#[cfg(windows)]
#[inline(always)]
pub fn windows_setup() {
    use windows_sys::Win32::{
        Foundation::{GetLastError, INVALID_HANDLE_VALUE},
        System::{
            Console::{
                GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
                STD_OUTPUT_HANDLE,
            },
            Threading::ExitProcess,
        },
    };
    unsafe {
        let std_output = GetStdHandle(STD_OUTPUT_HANDLE);
        if std_output == INVALID_HANDLE_VALUE {
            eprintln!("Error: stdout is invalid");
            ExitProcess(GetLastError());
        }
        let mut console_mode = 0;
        if GetConsoleMode(std_output, &mut console_mode) == 0 {
            eprintln!("Error: failed to get console mode");
            ExitProcess(GetLastError());
        }
        console_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        if SetConsoleMode(std_output, console_mode) == 0 {
            eprintln!("Error: failed to set console mode");
            ExitProcess(GetLastError());
        }
    }
}
