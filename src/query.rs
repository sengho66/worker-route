use crate::error::Error;
use crate::utils::struct_fields;

use serde::Deserialize;
use std::fmt::Debug;
use worker::Method;
use worker::Request;
use worker::RouteContext;

/// Extract typed information with the supplied struct from the query string from `Request`
/// To extract information from `Request`, `T` must implement `Deserialize` trait.
///
/// # Panics
/// If there's missing field from the URL query string and if the field
/// is not an `Option<T>` then it will panic.
///
/// # Examples
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use worker::{Response, Result, RouteContext};
/// use worker_route::{get, Query};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Foo {
///     foo: String,
/// }
/// // "/foo" will panic
/// // "/foo?foo=foo" will not
/// #[get("/foo")]
/// async fn foo(req: Query<Foo>, _: RouteContext<()>) -> Result<Response> {
///     // call req.into_inner() to access the field
///     Response::empty()
/// }
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct OptionFoo {
///     foo: Option<String>,
/// }
///
/// // "/foo" will not panic because foo is an `Option<T>`
/// #[get("/foo")]
/// async fn option_foo(req: Query<OptionFoo>, _: RouteContext<()>) -> Result<Response> {
///     Response::empty()
/// }
///
/// ```
///
pub struct Query<T>(T);
impl<T> Query<T>
where
    T: for<'a> Deserialize<'a> + Debug,
{
    /// Acess the owned `T`
    pub fn into_inner(self) -> T {
        self.0
    }

    fn collect_fields(fields: &'static [&'static str], ctx: &RouteContext<()>) -> Vec<String> {
        let mut map = Vec::with_capacity(fields.len());
        for i in fields {
            if let Some(p) = ctx.param(i) {
                map.push(format!("{}={}", i.trim(), p.trim()));
            }
        }

        map
    }

    #[allow(unused)]
    // method is unusedf or now
    fn new(method: Option<Method>, req: &Request, ctx: &RouteContext<()>) -> Result<Self, Error> {
        // get the fields from the supplied <T> which is a struct
        // struct Foo { name: String, age: usize }
        // ["name", "age"]
        let fields = struct_fields::<T>();
        let url = req.url().unwrap();

        // get fields from path first
        // "/my_path/:name/:age"
        let mut paths = Self::collect_fields(fields, ctx);

        // if the given route is "/my_path/{some_params}" then paths.len() should be empty
        // or if the given route is "/my_path/:name/{some_optional_params}"
        // then we try getting them from the url instead
        if paths.len() != fields.len() {
            if let Some(query) = url.query() {
                paths.push(query.to_owned())
            }
        }

        // ["name=Foo", "age=20"] becomes "name=Foo&age=20"
        let queries = paths.join("&");
        let params = serde_qs::from_str::<T>(&queries)?;

        Ok(Self(params))
    }

    /// Deserialize the given `T` from the URL query string.
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use worker::{console_log, Request, Response, Result, RouteContext};
    /// use worker_route::{get, Query};
    ///
    /// #[derive(Debug, Deserialize, Serialize)]
    /// struct Person {
    ///     name: String,
    ///     age: usize,
    /// }
    ///
    /// #[get("/persons/:name/:age")]
    /// async fn person(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    ///     let person = Query::<Person>::from(&req, &ctx);
    ///     let Person { name, age } = person.unwrap().into_inner();
    ///     console_log!("name: {name}, age: {age}");
    ///     Response::empty()
    /// }
    ///
    /// ```
    pub fn from(req: &Request, ctx: &RouteContext<()>) -> Result<Self, Error> {
        Self::new(None, req, ctx)
    }
    #[doc(hidden)]
    pub fn from_method(
        method: Method,
        req: &Request,
        ctx: &RouteContext<()>,
    ) -> Result<Self, Error> {
        Self::new(Some(method), req, ctx)
    }
}
