use std::future::Future;
use worker::{console_debug, Method, Request, Response, Result as CfResult, RouteContext, Router};

type Cb<D> = fn(Router<'static, D>) -> Router<'static, D>;

#[allow(unused)]
#[doc(hidden)]
pub struct RouteHandler<D, U> {
    fn_: fn(Request, RouteContext<D>) -> U,
    is_async: bool,
    method: Method,
    pattern: &'static str,
}

#[doc(hidden)]
impl<D, U> RouteHandler<D, U> {
    pub fn new(
        fn_: fn(Request, RouteContext<D>) -> U,
        pattern: &'static str,
        is_async: bool,
        method: Method,
    ) -> Self {
        Self {
            fn_,
            pattern,
            is_async,
            method,
        }
    }
}

/// A trait that's implemented for `RouteHandler` that you can call to configure
/// the routes and pattern.
///
/// # Example
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use worker::{event, Env, Request, Response, ResponseBody, Result, RouteContext, Router};
/// use worker_route::{get, AddRoute, Configure, Query};
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
    fn configure(
        self,
        fns_: RouteFn<D, impl Future<Output = CfResult<Response>> + 'static>,
    ) -> Router<'static, D>;
}

type RouteFn<D, U> = fn() -> RouteHandler<D, U>;

impl<D: 'static> Configure<D> for Router<'static, D> {
    fn configure(
        self,
        fns_: RouteFn<D, impl Future<Output = CfResult<Response>> + 'static>,
    ) -> Router<'static, D> {
        let RouteHandler {
            fn_,
            pattern,
            method,
            ..
        } = fns_();
        // for now all non-async methods are not supported
        match method {
            Method::Head => self.head_async(pattern, fn_),
            Method::Get => self.get_async(pattern, fn_),
            Method::Post => self.post_async(pattern, fn_),
            Method::Put => self.put_async(pattern, fn_),
            Method::Patch => self.patch_async(pattern, fn_),
            Method::Delete => self.delete_async(pattern, fn_),
            Method::Options => self.options_async(pattern, fn_),
            _ => {
                // the method variant is passed from the macro module
                // it should not panic by right.
                console_debug!("{:?} is not supported.", method);
                panic!()
            }
        }
    }
}

/// A trait that's implemented for `Router` that allows you to declutter your main code.
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
/// fn init_routes(router: Router<'static, ()>) -> Router<'static, ()> {
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
pub trait Service<D> {
    fn service(self, fns_: Cb<D>) -> Router<'static, D>;
}

impl<D> Service<D> for Router<'static, D> {
    fn service(self, fns_: Cb<D>) -> Router<'static, D> {
        fns_(self)
    }
}
