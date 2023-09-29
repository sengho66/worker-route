use super::response::{HttpResponse, ResponseBuilder};
use super::HttpRequest;
use crate::error::accept_json;
use core::fmt;
use http::header::ToStrError;
use http::StatusCode;

/// Generate [`HttpResponse`](HttpResponse) for custom error implementations.
pub trait ResponseError: fmt::Debug + fmt::Display {
    /// Creates a `Response` from error.
    fn error_response(&self, req: HttpRequest) -> HttpResponse;
    /// Returns status code for an error.
    ///
    /// Defaults to 500 Internal Server Error.
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
    /// Get the underlying error message.
    fn description(&self) -> String;
}

impl ResponseError for worker::Error {
    fn error_response(&self, req: HttpRequest) -> HttpResponse {
        let mut res = ResponseBuilder::init();
        let status_code = self.status_code();
        if !(400..=599).contains(&status_code.as_u16()) {
            return res
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body("error status codes must be in the 400-599 range".to_owned());
        }

        if let Self::Json(_) = &self {
            accept_json(&req, res.headers_mut());
        }

        res.status(status_code).body(self.description())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            worker::Error::Json(json) => {
                StatusCode::from_u16(json.1).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            worker::Error::SerdeJsonError(serde) => match serde.classify() {
                serde_json::error::Category::Io => StatusCode::INTERNAL_SERVER_ERROR,
                _ => StatusCode::BAD_REQUEST,
            },
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn description(&self) -> String {
        if let Self::Json(e) = self {
            return e.0.clone();
        }

        self.to_string()
    }
}

impl ResponseError for serde::de::value::Error {
    fn error_response(&self, _: HttpRequest) -> HttpResponse {
        HttpResponse::empty()
    }

    fn description(&self) -> String {
        self.to_string()
    }
}

impl ResponseError for ToStrError {
    fn error_response(&self, _: HttpRequest) -> HttpResponse {
        HttpResponse::empty()
    }

    fn description(&self) -> String {
        self.to_string()
    }
}

impl ResponseError for serde_qs::Error {
    fn error_response(&self, _: HttpRequest) -> HttpResponse {
        HttpResponse::empty()
    }

    fn description(&self) -> String {
        self.to_string()
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}
