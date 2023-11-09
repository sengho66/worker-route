use std::fmt::Display;

use serde::{Serialize, Serializer};
use worker_route::{
    http::{
        StatusCode, {HttpResponse, ResponseBuilder},
    },
    ResponseError,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomError {
    message: String,
    #[serde(serialize_with = "as_u16")]
    status_code: StatusCode,
}

impl CustomError {
    pub fn new(message: String, status_code: StatusCode) -> Self {
        Self {
            message,
            status_code,
        }
    }
}

fn as_u16<S>(x: &StatusCode, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(x.as_u16())
}

impl ResponseError for CustomError {
    fn error_response(&self, _: worker_route::http::HttpRequest) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).json(self)
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn description(&self) -> String {
        self.message.clone()
    }
}

// required trait
impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// this trait is implemented for using `?` operator on any functions from worker itself
impl From<worker::Error> for CustomError {
    fn from(err: worker::Error) -> Self {
        Self {
            message: err.description(),
            status_code: err.status_code(),
        }
    }
}
