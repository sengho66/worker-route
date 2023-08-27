use serde::{Deserialize, Serialize};
use worker::{console_log, event, Env, Request, Response, Result, RouteContext, Router};
use worker_route::{get, post, Configure, Query, Service};

#[derive(Debug, Serialize, Deserialize)]
struct Bar {
    bar: String,
}

#[get("/bar")]
async fn bar(query: Query<Bar>, ctx: RouteContext<Data>) -> Result<Response> {
    console_log!("It works. \n{:?}", ctx.data);

    Response::from_json(&query.into_inner())
}

#[derive(Debug, Serialize, Deserialize)]
struct Foo {
    foo: String,
}

#[get("/foo")]
async fn foo(query: Query<Foo>, ctx: RouteContext<Data>) -> Result<Response> {
    console_log!("It works. \n{:?}", ctx.data);

    Response::from_json(&query.into_inner())
}

#[derive(Debug, Serialize, Deserialize)]
struct FooBar {
    foo: String,
    bar: String,
}

#[get("/foo-bar")]
async fn foo_bar(query: Query<FooBar>, _: Request, ctx: RouteContext<Data>) -> Result<Response> {
    console_log!("It works. \n{:?}", ctx.data);

    Response::from_json(&query.into_inner())
}

#[derive(Debug, Deserialize, Serialize)]
struct Person {
    name: String,
    age: usize,
}

#[post("/person/:name/:age")]
async fn person(query: Query<Person>, ctx: RouteContext<Data>) -> Result<Response> {
    console_log!("It works. \n{:?}", ctx.data);

    Response::from_json(&query.into_inner())
}

fn init_routes(router: Router<'static, Data>) -> Router<'static, Data> {
    router
        .configure(bar)
        .configure(foo)
        .configure(person)
        .configure(foo_bar)
}

#[allow(unused)]
#[derive(Debug)]
struct Data {
    tls_version: String,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let tls_version = req.cf().tls_version();
    let router = Router::with_data(Data { tls_version });

    router.service(init_routes).run(req, env).await
}
