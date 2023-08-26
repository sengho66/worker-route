//!
//! Add handlers to CF Router with macro attribute.
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
//! // your function can consists of (Query<T>, Request, RouteContext<()>) too
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
//! fn configure(router: Router<'static, ()>) -> Router<'static, ()> {
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
//!     router.service(configure).run(req, env).await
//! }
//! ```
//!
//!
mod error;
mod query;
mod route;
mod utils;
mod wrapper;

pub use query::Query;
pub use route::{Configure, RouteHandler, Service};
pub use worker::Result as CfResult;
pub use worker_macro::{delete, get, head, options, patch, post, put};

#[doc(hidden)]
pub use wrapper::{_private_wrap, _private_wrap_with_query, _private_wrap_with_req};
