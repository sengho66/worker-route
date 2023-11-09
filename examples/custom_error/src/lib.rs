mod error;

use crate::error::CustomError;
use serde::Deserialize;
use worker::{event, Env, Request, Response, RouteContext, Router};
use worker_route::{get, http::StatusCode, Configure, Service};

#[allow(unused)]
#[derive(Deserialize)]
struct RandomStruct {
    foo: usize,
}

#[get("/error")]
fn error_(_: Request, _: RouteContext<()>) -> Result<Response, CustomError> {
    Err(CustomError::new(
        "Test".into(),
        StatusCode::INTERNAL_SERVER_ERROR,
    ))
}

#[get("/error-json")]
async fn question_mark_operator(
    mut req: Request,
    _: RouteContext<()>,
) -> Result<Response, CustomError> {
    // guaranteed to fail as it's a GET request
    _ = req.json::<RandomStruct>().await?;

    Ok(Response::empty()?)
}

fn init_routes(router: Router<'_, ()>) -> Router<'_, ()> {
    router.configure(question_mark_operator).configure(error_)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _: worker::Context) -> worker::Result<Response> {
    Router::new().service(init_routes).run(req, env).await
}
