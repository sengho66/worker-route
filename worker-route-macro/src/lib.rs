
extern crate proc_macro;
extern crate quote;
extern crate syn;
mod error;
mod expand;
mod route;
mod transform;

use proc_macro::TokenStream;
use route::Method;

macro_rules! route_method {
    ($variant:ident, $method:ident, $doc:expr, $doc1:expr) => {
        #[doc = concat!("A macro that creates route handler with `worker::Router::", stringify!($method), "_async`.")]
        ///
        /// # Usage
        /// ```text
        #[doc = concat!(" #[", stringify!($method), r#"("path")]"#)]
        /// ```
        ///
        /// # Attributes
        /// - `"path"`: Worker's path.
        ///
        /// # Examples
        /// ```
        /// use worker::{Result, Response};
        #[doc = $doc]
        ///
        #[doc = $doc1]
        /// async fn foo() -> Result<Response>{
        ///     Response::empty()
        /// }
        /// ```
        #[proc_macro_attribute]
        pub fn $method(attrs: TokenStream, items: TokenStream) -> TokenStream {
            route::new_route(attrs, items, Method::$variant)
        }
    }
}

route_method!(
    Delete,
    delete,
    "use worker_route::delete;",
    r#"#[delete("/path")]"#
);
route_method!(Head, head, "use worker_route::head;", r#"#[head("/path")]"#);
route_method!(
    Options,
    options,
    "use worker_route::options;",
    r#"#[options("/path")]"#
);
route_method!(
    Patch,
    patch,
    "use worker_route::patch;",
    r#"#[patch("/path")]"#
);
route_method!(Post, post, "use worker_route::post;", r#"#[post("/path")]"#);
route_method!(Put, put, "use worker_route::put;", r#"#[put("/path")]"#);
route_method!(Get, get, "use worker_route::get;", r#"#[get("/path")]"#);
