mod profile;
mod query;

use crate::query::{ProfileList, ProfileSingle};

use lazy_static::lazy_static;
use profile::Profile;
use serde::Serialize;
use worker::{event, Cors, Env, Method, Request, Response, Result, RouteContext, Router};
use worker_route::{get, Configure, Query, Service};

const PROFILES_BYTES: &[u8] = include_bytes!("profile.json");

lazy_static! {
    static ref PROFILE: Profile = serde_json::from_slice(PROFILES_BYTES).unwrap();
    static ref CUSTOM_CORS: Cors = {
        Cors::default()
            .with_origins(["*"])
            .with_allowed_headers(["*"])
            .with_methods(Method::all())
            .with_max_age(3600)
    };
}

// generic Json response
#[derive(Serialize)]
struct Res<T> {
    data: T,
}

// single path
// eg: /profile/Foo
#[get("/profile/:name", lazy_cors = CUSTOM_CORS)]
async fn get_single(query: Query<ProfileSingle>, ctx: RouteContext<Profile>) -> Result<Response> {
    let ProfileSingle { name } = query.into_inner();
    let res = ctx.data.single(&name.to_lowercase());

    Response::from_json(&Res { data: res })
}

// path with optional parameters
// eg: /profile?page=2&sort_by=email&order_by=desc
// defaults to page 1 if no parameters are present
#[get("/profile", lazy_cors = CUSTOM_CORS)]
async fn get_list(
    query: Query<ProfileList>,
    ctx: RouteContext<Profile>,
) -> std::result::Result<Response, worker::Error> {
    let ProfileList { page, sort } = query.into_inner();
    let mut profile = ctx.data;
    profile.filter(page, sort);

    Response::from_json(&Res { data: profile })
}

fn init_routes(router: Router<'_, Profile>) -> Router<'_, Profile> {
    router.configure(get_single).configure(get_list)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _: worker::Context) -> Result<Response> {
    Router::with_data(PROFILE.to_owned())
        .service(init_routes)
        .run(req, env)
        .await
}
