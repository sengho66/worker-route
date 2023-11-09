use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fmt::Display;
use worker::{event, Env, Request, Response, Result, RouteContext, Router};
use worker_route::http::header::ACCEPT;
use worker_route::http::{HttpRequest, HttpResponse, ResponseBuilder};
use worker_route::{get, Configure, Query, Responder, Service};

#[derive(Serialize, Deserialize)]
struct Person {
    name: String,
    age: usize,
}

#[allow(unused)]
#[derive(Deserialize, Serialize, Debug)]
struct CustomResponse {
    foo: String,
}

impl Display for CustomResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            &format!(r#"{{ "foo": {:?} }}"#, self.foo)
                .split_whitespace()
                .collect::<String>(),
        )
    }
}

impl Responder for CustomResponse {
    // HttpRequest is a cloned Request
    // below is an example of its basic use case
    fn to_response(self, req: HttpRequest) -> HttpResponse {
        let res = ResponseBuilder::init();
        if let Some(accept) = req.headers().get(&ACCEPT) {
            if accept.eq("*/*") | accept.eq("application/json") {
                return res.json(self);
            }
        }

        res.text(self.to_string())
    }
}

#[get("/bytes_response")]
fn bytes_response(_: Request, _: RouteContext<()>) -> Result<&'static [u8]> {
    // "Hello world." in bytes
    Ok(&[72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 46])
}

#[get("/serde_value_response/:name")]
fn serde_value_response(query: Query<Person>, _: RouteContext<()>) -> Result<Value> {
    Ok(json!({
        "name": query.name,
        "age": query.age
    }))
}

#[get("/string_response")]
fn string_response(_: Request, _: RouteContext<()>) -> Result<String> {
    Ok(String::from("Hello world."))
}

#[get("/struct_response")]
fn struct_response(_: Request, _: RouteContext<()>) -> Result<CustomResponse> {
    Ok(CustomResponse { foo: "Foo".into() })
}

fn init_routes(router: Router<'_, ()>) -> Router<'_, ()> {
    router
        .configure(bytes_response)
        .configure(serde_value_response)
        .configure(string_response)
        .configure(struct_response)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _: worker::Context) -> Result<Response> {
    Router::new().service(init_routes).run(req, env).await
}
