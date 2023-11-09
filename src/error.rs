use crate::http::{
    ContentType, ResponseError, {HttpRequest, HttpResponse, ResponseBuilder},
};
use core::fmt::{Debug, Display};
use http::{
    header::{ToStrError, ACCEPT, CONTENT_TYPE},
    StatusCode,
};
use mime::STAR_STAR;
use serde_json::{json, Value};
use worker::Headers;

/// Top level Worker-Route Error.
#[derive(Debug)]
pub struct Error {
    message: String,
    status_code: StatusCode,
    cause: ErrorCause,
}

/// All possible Error variants that may occur when working with [`worker_route`](crate).
#[derive(Debug)]
pub enum ErrorCause {
    /// Errors occured from [`worker::Error`](https://docs.rs/worker/latest/worker/enum.Error.html)
    Worker(worker::Error),
    /// Errors occured from [`Query`](crate::Query)
    Query,
    /// Errors occured from [`HttpHeaders`](crate::http::headers::HttpHeaders) operations
    Header,
    /// Errors occured from [`ResponseBuilder`](crate::http::ResponseBuilder)
    Json,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error {
    /// Creates a new [`Error`].
    pub(super) fn new(message: String, status_code: StatusCode, cause: ErrorCause) -> Self {
        Self {
            message,
            status_code,
            cause,
        }
    }

    /// Returns the underlying error's occurrence
    pub fn cause(&self) -> &ErrorCause {
        &self.cause
    }

    /// Get the underlying error message.
    pub fn description(&self) -> String {
        <Self as ResponseError>::description(self)
    }

    pub(super) fn to_error(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code).body(self.to_json())
    }

    pub(super) fn to_json(&self) -> Value {
        json!({
            "message": self.message,
            "statusCode": self.status_code.as_u16(),
            "success": false
        })
    }
}

impl ResponseError for Error {
    fn error_response(&self, req: HttpRequest) -> HttpResponse {
        let mut res = self.to_error();
        accept_json(&req, res.0.headers_mut());
        res
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn description(&self) -> String {
        self.message.clone()
    }
}

pub fn accept_json(req: &HttpRequest, headers: &mut Headers) {
    if let Some(accept) = req.headers().get(&ACCEPT) {
        if accept.contains(STAR_STAR.essence_str()) || accept.contains(ContentType::json().as_str())
        {
            _ = headers.set(
                CONTENT_TYPE.as_str(),
                ContentType::json().to_header_value().to_str().unwrap(),
            );
        }
    }
}

impl From<Error> for worker::Error {
    fn from(err: Error) -> Self {
        Self::Json((err.to_json().to_string(), err.status_code.into()))
    }
}

impl From<serde_qs::Error> for Error {
    fn from(err: serde_qs::Error) -> Self {
        Self {
            message: err.description(),
            status_code: err.status_code(),
            cause: ErrorCause::Query,
        }
    }
}

impl From<worker::Error> for Error {
    fn from(err: worker::Error) -> Self {
        Self {
            message: err.description(),
            status_code: err.status_code(),
            cause: ErrorCause::Worker(err),
        }
    }
}

impl From<serde::de::value::Error> for Error {
    fn from(err: serde::de::value::Error) -> Self {
        Self {
            message: err.description(),
            status_code: err.status_code(),
            cause: ErrorCause::Query,
        }
    }
}

impl From<ToStrError> for Error {
    fn from(err: ToStrError) -> Self {
        Self {
            message: err.description(),
            status_code: err.status_code(),
            cause: ErrorCause::Header,
        }
    }
}
