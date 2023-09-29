use worker::Request;

/// A handler middleware provides an access to [`worker::Request`](https://docs.rs/worker/latest/worker/struct.Request.html)
///
/// Currently this is only used to return [`Cors`](https://docs.rs/worker/latest/worker/struct.Cors.html)
///
/// # Examples
/// ```
/// use worker::{Cors, Request, Response, Result, RouteContext};
/// use worker_route::{route, Wrap};
///
/// // Doesn't necessarily have to be a unit struct.
/// // It can be anything.
/// pub struct MyCors;
///
/// impl Wrap for MyCors {
///     type Output = Cors;
///
///     fn wrap(req: &Request) -> Self::Output {
///         Cors::default()
///     }
/// }
///
/// #[route("/hello-world", method = "get", cors = MyCors)]
/// fn hello_world(req: Request, ctx: RouteContext<()>) -> Result<String> {
///     Ok("Hello world.".to_owned())
/// }
/// ```
pub trait Wrap {
    /// The output of the return value.
    type Output;

    fn wrap(req: &Request) -> Self::Output;
}
