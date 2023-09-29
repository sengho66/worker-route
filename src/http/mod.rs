//! Various HTTP related types
#![allow(
    clippy::use_self,
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn
)]
mod body;
mod content_type;
mod error;
pub(crate) mod headers;
mod impl_trait;
mod request;
mod responder;
mod response;

pub use body::Body;
pub use content_type::ContentType;
pub use error::ResponseError;
pub use headers::HttpHeaders;
pub use http::{header, StatusCode};
pub use request::HttpRequest;
pub(crate) use responder::InternalResponder;
pub use responder::Responder;
pub use response::{HttpResponse, ResponseBuilder};

#[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
#[cfg(feature = "cookies")]
pub(crate) mod cookies;

#[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
#[cfg(feature = "cookies")]
pub use cookie::Cookie;
