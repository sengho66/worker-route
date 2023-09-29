use super::HttpHeaders;

use worker::{Method, Request, Url};

/// Extracted from [`worker::Request`](https://docs.rs/worker/latest/worker/struct.Request.html) mainly used for [`Responder`](crate::Responder) trait.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct HttpRequest {
    headers: HttpHeaders,
    method: Method,
    path: String,
    url: Option<Url>,
}

impl HttpRequest {
    /// Returns the cloned request's headers.
    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    /// Request method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// The path of this request.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The parsed [`Url`] of this `Request`.
    ///
    /// None if errors occured from parsing the `Url`.
    pub fn url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
    #[cfg(feature = "cookies")]
    /// Request cookies.
    pub fn cookies(&self) -> impl Iterator<Item = cookie::Cookie<'_>> {
        use crate::http::cookies::CookieHelper;
        CookieHelper::Get.get(&self.headers)
    }
}

impl From<&Request> for HttpRequest {
/// This is constructed from code generation. Not a public method.
    fn from(req: &Request) -> Self {
        Self {
            headers: req.headers().into(),
            method: req.method(),
            path: req.path(),
            url: req.url().ok(),
        }
    }
}
