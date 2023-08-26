use serde::{Deserialize, Serialize};
use worker::{event, Env, Request, Response, Result, RouteContext, Router};
use worker_route::{get, post, Configure, Query, Service};

#[derive(Debug, Serialize, Deserialize)]
struct Bar {
    bar: String,
}

#[get("/bar")]
async fn bar(req: Query<Bar>, _: RouteContext<()>) -> Result<Response> {
    Response::from_json(&req.into_inner())
}

#[derive(Debug, Serialize, Deserialize)]
struct Foo {
    foo: String,
}

#[get("/foo")]
async fn foo(req: Query<Foo>, _: RouteContext<()>) -> Result<Response> {
    Response::from_json(&req.into_inner())
}
#[derive(Debug, Serialize, Deserialize)]

struct FooBar {
    foo: String,
    bar: String,
}

#[get("/foo-bar")]
async fn foo_bar(req: Query<FooBar>, _: Request, _: RouteContext<()>) -> Result<Response> {
    Response::from_json(&req.into_inner())
}

#[derive(Debug, Deserialize, Serialize)]
struct Person {
    name: String,
    age: usize,
}

#[post("/person/:name/:age")]
async fn person(req: Query<Person>, _: RouteContext<()>) -> Result<Response> {
    Response::from_json(&req.into_inner())
}

fn init_routes(router: Router<'static, ()>) -> Router<'static, ()> {
    router
        .configure(bar)
        .configure(foo)
        .configure(person)
        .configure(foo_bar)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let router = Router::new();
    router.service(init_routes).run(req, env).await
}
