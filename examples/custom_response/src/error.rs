use std::fmt::Display;

use serde::Serialize;
use worker::{console_log, Response};
use worker_route::{http::response::HttpResponse, HttpRequest, ResponseError};

#[derive(Debug, Serialize)]
pub struct CustomError {
    message: String,
    status_code: u16,
}

impl CustomError {
    pub fn new(message: impl Into<String>, status_code: u16) -> Self {
        Self {
            message: message.into(),
            status_code,
        }
    }
}

impl ResponseError for CustomError {
    fn error_response(&self, req: HttpRequest) -> HttpResponse {
        Response::from_json(self)
            .unwrap()
            .with_status(self.status_code)
            .into()
    }

    fn status_code(&self) -> u16 {
        self.status_code
    }

    fn description(&self) -> String {
        self.message.clone()
    }
}

// required trait
impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message.as_str())
    }
}

// this trait is implemented for using `?` operator on any functions from worker itself
impl From<worker::Error> for CustomError {
    fn from(err: worker::Error) -> Self {
        console_log!("HI worker");
        Self {
            message: err.to_string(),
            status_code: err.status_code(),
        }
    }
}

// this trait required in order to use `?` operator on any function from worker_route
// such as extracting query parameters from Query
impl From<worker_route::Error> for CustomError {
    fn from(err: worker_route::Error) -> Self {
        console_log!("HI routes");

        Self {
            message: err.description(),
            status_code: err.status_code(),
        }
    }
}
