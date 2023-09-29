//!
//! Worker Route is a crate designed for usage in Cloudflare Workers.
//!
//! # Examples
//! ```
//! use serde::{Deserialize, Serialize};
//! use worker::{event, Env, Request, Response, Result, RouteContext, Router};
//! use worker_route::{get, Configure, Query, Service};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Bar {
//!     bar: String,
//! }
//!
//! #[get("/bar")]
//! async fn bar(req: Query<Bar>, _: RouteContext<()>) -> Result<Response> {
//!     Response::from_json(&req.into_inner())
//! }
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Foo {
//!     foo: String,
//! }
//!
//! #[get("/foo")]
//! async fn foo(req: Query<Foo>, _: RouteContext<()>) -> Result<Response> {
//!     Response::from_json(&req.into_inner())
//! }
//!
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct FooBar {
//!     foo: String,
//!     bar: String,
//! }
//!
//! // your function can consist of (Query<T>, Request, RouteContext<()>) too
//! #[get("/foo-bar")]
//! async fn foo_bar(req: Query<FooBar>, _req: Request, _: RouteContext<()>) -> Result<Response> {
//!     Response::from_json(&req.into_inner())
//! }
//!
//! #[derive(Debug, Deserialize, Serialize)]
//! struct Person {
//!     name: String,
//!     age: usize,
//! }
//!
//! #[get("/person/:name/:age")]
//! async fn person(req: Query<Person>, _: RouteContext<()>) -> Result<Response> {
//!     Response::from_json(&req.into_inner())
//! }
//!
//! fn init_routes(router: Router<'_, ()>) -> Router<'_, ()> {
//!     router
//!         .configure(bar)
//!         .configure(foo)
//!         .configure(person)
//!         .configure(foo_bar)
//! }
//!
//! #[event(fetch)]
//! pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
//!     let router = Router::new();
//!     router.service(init_routes).run(req, env).await
//! }
//! ```
//!
//! # Features
//! - Add routes to handler with macro attribute
//! - Extract query parameters or path from URL
//!
#![warn(clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::use_self,
    clippy::similar_names,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::return_self_not_must_use
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
pub mod http;
mod middleware;
mod query;
mod route;
mod utils;

#[doc(hidden)]
mod internal;

pub use crate::http::{HttpHeaders, HttpRequest, HttpResponse, Responder, ResponseError};
pub use error::{Error, ErrorCause};
pub use middleware::Wrap;
pub use query::Query;
pub use route::{Configure, Service};
pub use worker_route_macro::{delete, get, head, options, patch, post, put, route};

#[doc(hidden)]
pub mod __private {
    pub use crate::internal::{respond_async, responder, FnType};
    pub use crate::route::{AddHandler, RouteFactory};
}
