use super::HttpRequest;

use super::response::{HttpResponse, ResponseBuilder};
use serde_json::Value;
use std::borrow::Cow;
use worker::{Cors, Response};

/// A worker's custom response implementation.
///
/// # Examples
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use worker::{Request, Response, RouteContext};
/// use worker_route::{get, HttpResponse, HttpRequest, http::ResponseBuilder, Responder};
///
/// #[allow(unused)]
/// #[derive(Deserialize, Serialize)]
/// struct Foo {
///     foo: String,
/// }
///
/// impl Responder for Foo {
///     fn to_response(self, _req: HttpRequest) -> HttpResponse {
///         ResponseBuilder::init().json(self)
///     }
/// }
///
/// #[get("/custom_response")]
/// async fn custom_response(_: Request, _: RouteContext<()>) -> Result<Foo, worker::Error> {
///     Ok(Foo { foo: String::from("Bar") })
/// }
/// ```
///
pub trait Responder {
    /// Convert `Self` to [`HttpResponse`]
    fn to_response(self, req: HttpRequest) -> HttpResponse;
}

pub trait InternalResponder {
    fn into_response(self, cors: Option<&Cors>) -> HttpResponse;
}

macro_rules! with_cors {
    ($self_:expr, $cors:ident, $expr:expr) => {
        if $cors.is_none() {
            return $expr;
        }
        $self_.with_cors($cors.unwrap()).into()
    };
}

impl InternalResponder for HttpResponse {
    fn into_response(self, cors: Option<&Cors>) -> HttpResponse {
        with_cors! {
            self.into_res(), cors, self
        }
    }
}

impl InternalResponder for Response {
    fn into_response(self, cors: Option<&Cors>) -> HttpResponse {
        with_cors! {
            self, cors, self.into()
        }
    }
}

impl Responder for HttpResponse {
    fn to_response(self, _: HttpRequest) -> HttpResponse {
        self
    }
}

impl Responder for Response {
    fn to_response(self, _: HttpRequest) -> HttpResponse {
        self.into()
    }
}

macro_rules! impl_responder_for {
    () => {
        ResponseBuilder::init()
    };
    ($res:ty, $expr:tt) => {
        impl Responder for $res {
            fn to_response(self, _: HttpRequest) -> HttpResponse {
                impl_responder_for!().$expr(self)
            }
        }
    };
    ($res:ty, $expr:tt, $ops:tt) => {
        impl Responder for $res {
            fn to_response(self, _: HttpRequest) -> HttpResponse {
                impl_responder_for!().$expr(self.$ops())
            }
        }
    };
}

impl_responder_for!(Value, json);
impl_responder_for!(String, text);
impl_responder_for!(Cow<'_, str>, text);
impl_responder_for!(&'static str, text);
impl_responder_for!(Vec<u8>, bytes);
impl_responder_for!(&'static [u8], bytes, into);
