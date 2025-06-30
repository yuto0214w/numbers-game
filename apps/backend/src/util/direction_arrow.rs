use std::fmt;

use axum::http::StatusCode;

use super::WebSocketAction;

const DIRECTION_ARROWS: &'static str = "<->-x--v";

#[derive(Clone, Copy)]
pub enum DirectionArrow {
    RTL,
    LTR,
    ErrorLTR = 3,
    ErrorRTL,
    NoConnection,
    Redirect,
}

impl DirectionArrow {
    pub fn as_str(&self) -> &'static str {
        let index = *self as usize;
        unsafe { DIRECTION_ARROWS.get_unchecked(index..index + 2) }
    }
}

impl fmt::Display for DirectionArrow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&StatusCode> for DirectionArrow {
    fn from(value: &StatusCode) -> Self {
        let code = value.as_u16();
        if code >= 500 {
            Self::ErrorRTL
        } else if code >= 400 {
            Self::ErrorLTR
        } else if code >= 300 {
            Self::Redirect
        } else {
            Self::LTR
        }
    }
}

impl From<&WebSocketAction<'_>> for DirectionArrow {
    fn from(value: &WebSocketAction<'_>) -> Self {
        match value {
            WebSocketAction::Connect => Self::LTR,
            WebSocketAction::Disconnect => Self::NoConnection,
            WebSocketAction::Receive(_) => Self::LTR,
            WebSocketAction::Send(Ok(_)) => Self::RTL,
            WebSocketAction::Send(Err(_)) => Self::ErrorRTL,
        }
    }
}
