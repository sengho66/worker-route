#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::use_self)]
extern crate proc_macro;
extern crate quote;
extern crate syn;

mod error;
mod expand;
mod method;
mod route;
mod transform;
mod wrapper;
use method::Method;
use paste::paste;
use proc_macro::TokenStream;

macro_rules! route_method {
    ($variant:ident, $method:ident) => {
        paste! {
            #[doc = "A macro that creates route handler with [`worker::Router::" $method "" "`]" "(https://docs.rs/worker/latest/worker/struct.Router.html#method." "" $method """)"]
            ///
            #[doc = "[`worker::Router::" $method "_async" "`]" "(https://docs.rs/worker/latest/worker/struct.Router.html#method." "" $method "_async"") will be used if the handler is an async fn."]
            ///
            /// # Usage
            /// ```text
            #[doc = " #[" $method "" r#"("/path")]"#]
            /// ```
            ///
            /// # Attributes
            /// - `"path"`: Worker's path.
            /// - `Option<cors>`: Wrap a struct that implements `worker_route::MwService`.
            /// - `Option<lazy_cors>`: Wrap a lazy initialized Cors.
            /// - `Option<wrap>`: Register an options handler with the provided cors. Defaults to `None`.
            ///
            /// # Examples
            /// ```
            /// use worker::{Result, Request, RouteContext, Response};
            #[doc = "use worker_route::" $method ";"]
            ///
            #[doc = "#[" $method "" r#"("/path")]"#]
            /// async fn foo(req: Request, ctx: RouteContext<()>) -> Result<Response>{
            ///     Response::empty()
            /// }
            /// ```
            #[proc_macro_attribute]
            pub fn $method(attrs: TokenStream, items: TokenStream) -> TokenStream {
                route::with_method::<{ Method::$variant as _ }>(attrs, items)
            }
        }
    }
}

/// A macro that creates route handler with multiple methods.
///
/// # Usage
/// ```text
/// #[route("path", method = "method", cors = "cors", lazy_cors = "lazy_cors", wrap)]
/// ```
///
/// # Attributes
/// - `"path"`: Worker's path.
/// - `method`: An array of methods or a method in string literal.
/// - `Option<cors>`: Wrap a struct that implements `worker_route::MwService`.
/// - `Option<lazy_cors>`: Wrap a lazy initialized Cors.
/// - `Option<wrap>`: Register an options handler with the provided cors. Defaults to `None`.
///
/// # Examples
/// ```
/// use worker::{Result, Request, RouteContext, Response, route};
///
/// #[route("/path", method = "get", method = "post")]
/// async fn foo(req: Request, ctx: RouteContext<()>) -> Result<Response> {
///     Response::empty()
/// }
/// ```
#[proc_macro_attribute]
pub fn route(attrs: TokenStream, items: TokenStream) -> TokenStream {
    route::with_method::<{ Method::Default as _ }>(attrs, items)
}

route_method!(Delete, delete);
route_method!(Get, get);
route_method!(Head, head);
route_method!(Options, options);
route_method!(Patch, patch);
route_method!(Post, post);
route_method!(Put, put);
