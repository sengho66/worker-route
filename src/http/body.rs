use super::content_type::ContentType;
use super::response::ResponseBuilder;
use crate::error::Error;
use crate::ErrorCause;
use http::StatusCode;
use serde_json::Value;
use worker::worker_sys::web_sys::ReadableStream;
use worker::ResponseBody;

/// A wrapper for [`worker::ResponseBody`](https://docs.rs/worker/latest/worker/enum.ResponseBody.html).
///
/// The `Body` variant differs from `ResponseBody` body variant, this stores a [`Box<[u8]>`](Box) instead of a [`Vec<u8>`].
#[derive(Debug)]
pub enum Body {
    Empty,
    Body(Box<[u8]>),
    Stream(ReadableStream),
}

type JsonSerialize = Result<Vec<u8>, serde_json::Error>;

pub enum SetBody {
    Body(ResponseBody),
    Json(JsonSerialize),
}

impl SetBody {
    fn body(builder: ResponseBuilder, body: Body) -> ResponseBuilder {
        ResponseBuilder { body, ..builder }
    }

    pub fn set_body(self, builder: ResponseBuilder) -> ResponseBuilder {
        match self {
            Self::Body(body) => Self::body(builder, body.into()),
            Self::Json(v) => Self::json(builder, v),
        }
    }

    fn json(mut builder: ResponseBuilder, value: JsonSerialize) -> ResponseBuilder {
        match value {
            Ok(v) => {
                let bytes = Box::<[u8]>::from(v.as_slice());
                builder.set_content_type(&ContentType::json());
                builder.body = Body::Body(bytes);
            }
            Err(e) => {
                let status = StatusCode::INTERNAL_SERVER_ERROR;
                builder.error = Some(Error::new(e.to_string(), status, ErrorCause::Json));
                builder.status = Some(status);
            }
        }

        builder
    }
}

macro_rules! set_body {
    ($b:tt, $val:expr, $self_:ident) => {
        SetBody::$b($val).set_body($self_)
    };
    ($b:tt, $val:expr, $self_:ident, $content:ident) => {
        set_body!($b, ResponseBody::Body($val), $self_)
            .content_type(&ContentType::$content())
            .take()
    };
}

pub(super) use set_body;

macro_rules! impl_for_body {
    ($res:ty, $($expr:tt)*) => {
        impl From<$res> for Body {
            fn from(b: $res) -> Self {
                Self::Body(b.$($expr)*)
            }
        }
    };
}

macro_rules! from_body {
    ($from:tt) => {
        from_body!(&$from, Body, $from, b, b.as_slice().into(), s, s.clone());
    };
    ($from:tt, $to:ty) => {
        from_body!($from, $to, $from, b, b.as_slice().into(), s, s);
    };
    ($from:ty, $to:ty, $from_:tt) => {
        from_body!($from, $to, $from_, b, b.into(), s, s);
    };
    ($from:ty, $to:ty, $from_:tt, $stream:ident, $s:expr) => {
        from_body!($from, $to, $from_, b, b.as_slice().into(), $stream, $s);
    };  
    ($from:ty, $to:ty, $from_:tt, $b:ident, $b_:expr, $stream:ident, $s:expr) => {
        impl From<$from> for $to {
            fn from(b: $from) -> Self {
                match b {
                    $from_::Empty => Self::Empty,
                    $from_::Body($b) => Self::Body($b_),
                    $from_::Stream($stream) => Self::Stream($s),
                }
            }
        }
    };
}

from_body!(ResponseBody);
from_body!(ResponseBody, Body);
from_body!(Body, ResponseBody, Body);
impl_for_body!(String, as_bytes().into());
impl_for_body!(Vec<u8>, as_slice().into());
impl_for_body!(&'static [u8], into());
impl_for_body!(&'static str, as_bytes().into());
impl_for_body!(Value, to_string().as_bytes().into());
