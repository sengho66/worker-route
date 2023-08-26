use serde::Serialize;
use serde_json::to_value;
use std::fmt::{Debug, Display};
use worker::{Response, Result as CfResult};

#[derive(Serialize, Clone, Debug)]
pub struct ErrorJson {
    status: u16,
    message: String,
}

#[derive(Debug)]
pub struct Error(worker::Error);

impl ErrorJson {
    fn new(message: String, status: u16) -> Self {
        ErrorJson { status, message }
    }
}

impl Error {
    pub fn new(v: String) -> Self {
        Self(worker::Error::from(v))
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        match value.io_error_kind() {
            Some(s) => Self(worker::Error::from(s.to_string())),
            None => Self(worker::Error::from(value.to_string())),
        }
    }
}

impl From<Error> for worker::Error {
    fn from(value: Error) -> Self {
        value.0
    }
}

impl From<Error> for ErrorJson {
    fn from(value: Error) -> Self {
        match value.0 {
            worker::Error::Json(json) => Self::new(json.0, json.1),
            _ => Self::new(value.0.to_string(), 404u16),
        }
    }
}

impl From<ErrorJson> for String {
    fn from(value: ErrorJson) -> Self {
        to_value(value).unwrap().to_string()
    }
}

impl From<serde_qs::Error> for Error {
    fn from(value: serde_qs::Error) -> Self {
        Self(worker::Error::from(value.to_string()))
    }
}

impl From<worker::Error> for ErrorJson {
    fn from(value: worker::Error) -> Self {
        match value {
            worker::Error::Json(json) => Self::new(json.0, json.1),
            _ => Self::new(value.to_string(), 404u16),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for ErrorJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ErrorJson {{ message: {}, status: {} }}",
            self.message, self.status
        )
    }
}

impl From<ErrorJson> for CfResult<Response> {
    fn from(value: ErrorJson) -> CfResult<Response> {
        Response::error(value.message, value.status)
    }
}
