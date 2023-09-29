use core::future::Future;
use worker::{console_debug, Method, Request, Response, Result, RouteContext, Router};

// This trait is exactly the same as the one that RouteContext uses to get params
// This is used mainly for testing-suite
pub trait Params {
    fn param_(&self, key: &str) -> Option<&String>;
}

impl<D> Params for RouteContext<D> {
    fn param_(&self, key: &str) -> Option<&String> {
        self.param(key)
    }
}

#[doc(hidden)]
#[allow(clippy::module_name_repetitions)]
/// Used for code generation, not for public usage.
pub trait RouteFactory<D> {
    /// Used for code generation, not for public usage.
    fn register(self, router: Router<'_, D>) -> Router<'_, D>;
}

/// Implemented for [`worker::Router`](https://docs.rs/worker/latest/worker/struct.Router.html) to configure the route's pattern.
///
/// # Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use worker::{event, Env, Request, Response, ResponseBody, Result, RouteContext, Router};
/// use worker_route::{get, Configure, Query, Service};
///
/// #[derive(Debug, Deserialize, Serialize)]
/// struct Person {
///     name: String,
///     age: usize,
/// }
///
/// #[get("/person/:name/:age")]
/// async fn person(req: Query<Person>, _: RouteContext<()>) -> Result<Response> {
///     Response::from_json(&req.into_inner())
/// }
///
/// #[event(fetch)]
/// pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
///     let router = Router::new();
///     router.configure(person).run(req, env).await
/// }
///
/// ```
///
pub trait Configure<D> {
    fn configure<F: RouteFactory<D>>(self, f: F) -> Self;
}

impl<D> Configure<D> for Router<'_, D> {
    fn configure<F: RouteFactory<D>>(self, f: F) -> Self {
        f.register(self)
    }
}

/// Implemented for [`worker::Router`](https://docs.rs/worker/latest/worker/struct.Router.html) to run external route configuration.
/// 
/// This trait is useful for splitting the configuration to a different module.
///
/// # Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use worker::{event, Env, Request, Response, ResponseBody, Result, RouteContext, Router};
/// use worker_route::{get, Service, Configure, Query};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Bar {
///     bar: String,
/// }
///
/// #[get("/bar")]
/// async fn bar(req: Query<Bar>, _: RouteContext<()>) -> Result<Response> {
///     Response::from_body(ResponseBody::Body(req.into_inner().bar.as_bytes().into()))
/// }
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Foo {
///     foo: String,
/// }
///
/// #[get("/foo")]
/// async fn foo(req: Query<Foo>, _: RouteContext<()>) -> Result<Response> {
///     Response::from_body(ResponseBody::Body(req.into_inner().foo.as_bytes().into()))
/// }
///
/// #[derive(Debug, Deserialize, Serialize)]
/// struct Person {
///     name: String,
///     age: usize,
/// }
///
/// #[get("/person/:name/:age")]
/// async fn person(req: Query<Person>, _: RouteContext<()>) -> Result<Response> {
///     Response::from_json(&req.into_inner())
/// }
///
/// // wrapper function
/// fn init_routes(router: Router<'_, ()>) -> Router<'_, ()> {
///     router.configure(bar).configure(foo).configure(person)
/// }
///
/// #[event(fetch)]
/// pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
///     let router = Router::new();
///     // before
///     // router.configure(bar).configure(foo).configure(person).run(req, env).await
///     // after
///     // router.service(init_routes).run(req, env).await
///     router.service(init_routes).run(req, env).await
/// }
///
/// ```
pub trait Service {
    fn service<F: FnOnce(Self) -> Self>(self, f: F) -> Self
    where
        Self: Sized;
}

impl<D> Service for Router<'_, D> {
    fn service<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        f(self)
    }
}

type Handler<D, U> = fn(Request, RouteContext<D>) -> U;

#[doc(hidden)]
/// Used for code generation, not for public usage.
pub trait AddHandler<'a, D> {
    /// Used for code generation, not for public usage.
    fn register(
        self,
        pattern: &str,
        method: Method,
        sync_handler: Handler<D, Result<Response>>,
    ) -> Self;
    /// Used for code generation, not for public usage.
    fn register_async<U: Future<Output = Result<Response>> + 'a>(
        self,
        pattern: &str,
        method: Method,
        async_handler: Handler<D, U>,
    ) -> Self;
}

impl<'a, D: 'a> AddHandler<'a, D> for Router<'a, D> {
    fn register(
        self,
        pattern: &str,
        method: Method,
        sync_handler: Handler<D, Result<Response>>,
    ) -> Self {
        match method {
            Method::Head => self.head(pattern, sync_handler),
            Method::Get => self.get(pattern, sync_handler),
            Method::Post => self.post(pattern, sync_handler),
            Method::Put => self.put(pattern, sync_handler),
            Method::Patch => self.patch(pattern, sync_handler),
            Method::Delete => self.delete(pattern, sync_handler),
            Method::Options => self.options(pattern, sync_handler),
            _ => {
                // the method variant is passed from the macro module
                // it should not panic by right.
                console_debug!("{:?} is not supported.", method);
                panic!()
            }
        }
    }

    fn register_async<U: Future<Output = Result<Response>> + 'a>(
        self,
        pattern: &str,
        method: Method,
        async_handler: Handler<D, U>,
    ) -> Self {
        match method {
            Method::Head => self.head_async(pattern, async_handler),
            Method::Get => self.get_async(pattern, async_handler),
            Method::Post => self.post_async(pattern, async_handler),
            Method::Put => self.put_async(pattern, async_handler),
            Method::Patch => self.patch_async(pattern, async_handler),
            Method::Delete => self.delete_async(pattern, async_handler),
            Method::Options => self.options_async(pattern, async_handler),
            _ => {
                // the method variant is passed from the macro module
                // it should not panic by right.
                console_debug!("{:?} is not supported.", method);
                panic!()
            }
        }
    }
}
