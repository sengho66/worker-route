use super::body::set_body;
use super::body::{Body, SetBody};
use super::content_type::ContentType;
use super::headers::{HeadersOp, HttpHeaders};
use super::impl_trait::ToResponse;
use crate::error::Error;

use http::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use http::StatusCode;
use serde::Serialize;
use serde_json::to_vec;
use worker::worker_sys::web_sys::Response as SysResponse;
use worker::ResponseBody;
use worker::WebSocket;
use worker::{Cors, Response};

/// A wrapper for [`worker::Response`](https://docs.rs/worker/latest/worker/struct.Response.html).
///
/// By using `HttpResponse`, it allows you to work with with the response object without having to work with `Result` and unecessary unwrap.
#[derive(Debug)]
pub struct HttpResponse(pub(crate) Response);

impl HttpResponse {
    /// Constructs a response from [worker::Response](https://docs.rs/worker/latest/worker/struct.Response.html).
    pub fn from_response(res: Response) -> Self {
        res.into()
    }

    pub(crate) fn into_res(self) -> Response {
        self.into()
    }

    /// Constructs an empty response.
    pub fn empty() -> Self {
        Response::empty().into()
    }
}

/// An alternative [`worker::Response`](https://docs.rs/worker/latest/worker/struct.Response.html) builder.
///
/// # Examples
/// ```
/// use worker::{Request, RouteContext};
/// use worker_route::{get, http::ContentType, HttpResponse, http::ResponseBuilder};
///
/// #[get("/hello_world")]
/// fn hello_world(_: Request, _: RouteContext<()>) -> worker::Result<HttpResponse> {
///     Ok(ResponseBuilder::init()
///         .content_type(&ContentType::plaintext())
///         .body(String::from("Hello world.")))
/// }
/// ```
#[derive(Debug)]
pub struct ResponseBuilder {
    pub(super) body: Body,
    pub(super) error: Option<Error>,
    pub(super) headers: HttpHeaders,
    pub(super) status: Option<StatusCode>,
    pub(super) web_socket: Option<WebSocket>,
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self {
            body: Body::Empty,
            error: None,
            headers: HttpHeaders::default(),
            status: None,
            web_socket: None,
        }
    }
}

impl ResponseBuilder {
    /// Constructs a response builder.
    pub fn init() -> Self {
        Self::default()
    }

    /// Constructs a response builder with HTTP status.
    pub fn new(status: StatusCode) -> Self {
        Self::init().status(status)
    }

    /// Set response content type.
    pub fn content_type(mut self, v: &ContentType) -> Self {
        self.set_content_type(v);
        self
    }

    /// Set response status code.
    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = Some(status);
        self
    }

    /// Set response status code.
    pub fn set_status(&mut self, status: StatusCode) -> &mut Self {
        self.status = Some(status);
        self
    }

    /// Set a body to the response.
    pub fn body<B: Into<Body>>(self, body: B) -> HttpResponse {
        set_body!(Body, body.into().into(), self).take()
    }

    /// Set the bytes to the response body and `content-type` as `application/octet-stream`.
    pub fn bytes(self, bytes: Vec<u8>) -> HttpResponse {
        set_body!(Body, bytes, self, octet_stream)
    }

    /// Set the HTML content to the response body and `content-type` as `text/html`.
    pub fn html(self, html: &str) -> HttpResponse {
        set_body!(Body, html.into(), self, html)
    }

    /// Set the JSON to the response body and `content-type` as `application/json`.
    pub fn json<S: Serialize>(self, value: S) -> HttpResponse {
        set_body!(Json, to_vec(&value), self).take()
    }

    /// Set a body to the response.
    pub fn text<T: Into<String>>(self, body: T) -> HttpResponse {
        set_body!(Body, body.into().as_bytes().into(), self, plaintext)
    }

    pub(super) fn error(&mut self, err: Error) {
        let content_type = ContentType::plaintext();
        self.set_content_type(&content_type);
        self.error = Some(err);
    }

    /// Insert a header, replacing any that were set with an equivalent field name.
    ///
    /// # Examples
    /// ```
    /// use worker::{Request, RouteContext};
    /// use worker_route::{get, http};
    /// use http::{
    ///     header::{HeaderName, HeaderValue},
    ///     HttpResponse, ResponseBuilder, ContentType,
    /// };
    ///
    /// #[get("/hello_world")]
    /// fn hello_world(_: Request, _: RouteContext<()>) -> worker::Result<HttpResponse> {
    ///     let mut res = ResponseBuilder::init();
    ///     res.insert_header(
    ///         HeaderName::from_static("my-header"),
    ///         HeaderValue::from_static("my-value"),
    ///     );
    ///
    ///     Ok(res.body(String::from("Hello world.")))
    /// }
    ///
    /// ```
    ///
    /// # Errors
    ///
    /// Errors are returned to the response if the header name or value is invalid or contains empty spaces.
    ///
    /// # Panics
    ///
    /// Panics if [`HeaderName`] or [`HeaderValue`] is constructed from using the method [`from_static`](HeaderName::from_static)
    /// and the static string is an invalid header or contains spaces.
    pub fn insert_header(&mut self, k: HeaderName, v: HeaderValue) -> &mut Self {
        HeadersOp::Insert.set(&(k, v), self);
        self
    }

    /// Append a header, keeping any that were set with an equivalent field name.
    ///
    /// # Examples
    /// ```
    /// use http::{
    ///     header::{HeaderName, HeaderValue},
    ///     HttpResponse, ResponseBuilder, ContentType,
    /// };
    /// use worker::{Request, RouteContext};
    /// use worker_route::{get, http};
    ///
    /// #[get("/hello_world")]
    /// fn hello_world(_: Request, _: RouteContext<()>) -> worker::Result<HttpResponse> {
    ///     let mut res = ResponseBuilder::init();
    ///     res.append_header(
    ///         HeaderName::from_static("my-header"),
    ///         HeaderValue::from_static("my-value"),
    ///     );
    ///
    ///     Ok(res.body(String::from("Hello world.")))
    /// }
    ///
    /// ```
    ///
    /// # Errors
    ///
    /// Errors are returned to the response if the header name or value is invalid or contains empty spaces.
    ///
    /// # Panics
    ///
    /// Panics if [`HeaderName`] or [`HeaderValue`] is constructed from using the method [`from_static`](HeaderName::from_static)
    /// and the static string is an invalid header or contains empty spaces.
    pub fn append_header(&mut self, k: HeaderName, v: HeaderValue) -> &mut Self {
        HeadersOp::Append.set(&(k, v), self);
        self
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
    #[cfg(feature = "cookies")]
    /// Add a cookie to this response.
    pub fn add_cookie(&mut self, cookie: cookie::Cookie<'_>) -> &mut Self {
        use super::cookies::CookieHelper;
        CookieHelper::Set.set(self, &cookie);
        self
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
    #[cfg(feature = "cookies")]
    /// Get an iterator for the cookies set by this response.
    pub fn cookies(&self) -> impl Iterator<Item = cookie::Cookie<'_>> {
        use super::cookies::CookieHelper;
        CookieHelper::Set.get(&self.headers)
    }

    /// Set response content type.
    pub fn set_content_type(&mut self, v: &ContentType) -> &mut Self {
        if self.headers.get(&CONTENT_TYPE).is_none() {
            HeadersOp::Insert.set(&(CONTENT_TYPE, v.to_header_value()), self);
        }
        self
    }

    /// Sets this response's cors headers from the `Cors` struct.
    ///
    /// # Examples
    /// ```
    /// use worker::{Request, Method, RouteContext, Cors};
    /// use worker_route::{get, http::ContentType, HttpResponse, http::ResponseBuilder};
    ///
    /// #[get("/hello_world")]
    /// fn hello_world(_: Request, _: RouteContext<()>) -> worker::Result<HttpResponse> {
    ///     let mut res = ResponseBuilder::init();
    ///     let my_cors = Cors::default()
    ///        .with_origins(&["*".to_string()])
    ///        .with_allowed_headers(&["method".to_string(), "origin".to_string()])
    ///        .with_methods([Method::Get, Method::Options])
    ///        .with_max_age(86400);
    ///
    ///     res.with_cors(&my_cors);
    ///
    ///     Ok(res
    ///         .content_type(&ContentType::plaintext())
    ///         .body(String::from("Hello world.")))
    /// }
    /// ```
    pub fn with_cors(&mut self, cors: &Cors) -> &mut Self {
        if let Err(err) = self.headers_mut().apply_cors(cors) {
            self.error(err.into());

            return self;
        }

        self
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "experimental")))]
    #[cfg(feature = "experimental")]
    pub fn websocket(mut self, websocket: worker::WebSocket) -> Self {
        // make body empty as it should be
        if let Body::Stream(_) | Body::Body(_) = self.body {
            self.body = Body::Empty;
        }

        Self {
            status: Some(StatusCode::SWITCHING_PROTOCOLS),
            web_socket: Some(websocket),
            ..self
        }
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "experimental")))]
    #[cfg(feature = "experimental")]
    pub fn stream<S>(mut self, stream: S) -> Self
    where
        S: futures::TryStream + 'static,
        S::Ok: Into<Vec<u8>>,
        S::Error: Into<worker::Error>,
    {
        match Response::from_stream::<S>(stream) {
            Ok(res) => {
                self.body = res.body().into();
            }
            Err(err) => {
                self.status = Some(StatusCode::INTERNAL_SERVER_ERROR);
                self.error = Some(err.into());
            }
        }

        self
    }

    /// Read the [`HttpHeaders`] on this response.
    pub fn headers(&self) -> &HttpHeaders {
        &self.headers
    }

    /// Get a mutable reference to the `Headers` on this response.
    pub fn headers_mut(&mut self) -> &mut HttpHeaders {
        &mut self.headers
    }

    /// Read the [`StatusCode`] on this response.
    pub fn status_code(&self) -> Option<&StatusCode> {
        self.status.as_ref()
    }

    /// Set the [`StatusCode`] on this response.
    pub fn status_mut(&mut self) -> Option<&mut StatusCode> {
        self.status.as_mut()
    }

    fn take(mut self) -> HttpResponse {
        if let Some(err) = self.error.take() {
            return err.to_error().0.with_headers(self.headers.into()).into();
        }

        let res = match self.body {
            Body::Empty => self.web_socket.into_response(),
            Body::Body(b) => b.into_response(),
            Body::Stream(s) => s.into_response(),
        };

        res.with_headers(self.headers.into())
            .with_status(self.status.unwrap_or(StatusCode::OK).into())
            .into()
    }
}

impl From<Response> for HttpResponse {
    fn from(res: Response) -> Self {
        Self(res)
    }
}

impl From<HttpResponse> for SysResponse {
    fn from(res: HttpResponse) -> Self {
        res.0.into()
    }
}

impl From<worker::Result<Response>> for HttpResponse {
    fn from(res: worker::Result<Response>) -> Self {
        Self(res.into_response())
    }
}

impl From<HttpResponse> for worker::Result<Response> {
    fn from(res: HttpResponse) -> Self {
        Ok(res.0)
    }
}

impl From<HttpResponse> for Response {
    fn from(res: HttpResponse) -> Self {
        res.0
    }
}
