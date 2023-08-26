use serde::*;
use std::{fmt::Debug, future::Future};

use worker::{Method, Request, Response, Result as CfResult, RouteContext};

use crate::{
    error::{Error, ErrorJson},
    query::Query,
};

type QueryRes<T> = Result<Query<T>, Error>;
type WithQuery<T, U> = fn(Query<T>, RouteContext<()>) -> U;
type WithReq<T, U> = fn(Query<T>, Request, RouteContext<()>) -> U;
type Res = Result<CfResult<Response>, Error>;

enum FnType<T, U> {
    WithQuery(WithQuery<T, U>),
    WithReq(WithReq<T, U>),
}

trait Responder {
    fn res(self) -> CfResult<Response>;
}

impl Responder for Res {
    fn res(self) -> CfResult<Response> {
        match self {
            Ok(res) => res,
            Err(err) => ErrorJson::from(err).into(),
        }
    }
}

struct Wrapper<T, U> {
    ctx: RouteContext<()>,
    fn_: FnType<T, U>,
    req: Request,
    res: QueryRes<T>,
}

impl<T, U> Wrapper<T, U> {
    fn new(res: QueryRes<T>, req: Request, ctx: RouteContext<()>, fn_: FnType<T, U>) -> Self {
        Self { req, res, ctx, fn_ }
    }

    async fn res(self) -> Result<CfResult<Response>, Error>
    where
        T: for<'a> Deserialize<'a> + Debug + for<'a> serde::Deserialize<'a>,
        for<'a> U: Future<Output = CfResult<Response>> + 'a,
    {
        match self.res {
            Ok(query_) => match self.fn_ {
                FnType::WithQuery(fn_) => Ok(fn_(query_, self.ctx).await),
                FnType::WithReq(fn_) => Ok(fn_(query_, self.req, self.ctx).await),
            },
            Err(err) => Err(err),
        }
    }
}

#[allow(unused)]
pub async fn _private_wrap<U>(
    method: Method,
    req: Request,
    ctx: RouteContext<()>,
    fn_: fn(Request, RouteContext<()>) -> U,
) -> CfResult<Response>
where
    for<'a> U: Future<Output = CfResult<Response>> + 'a,
{
    match fn_(req, ctx).await {
        Ok(query_) => Ok(query_),
        Err(err) => ErrorJson::from(err).into(),
    }
}

pub async fn _private_wrap_with_query<T, U>(
    method: Method,
    req: Request,
    ctx: RouteContext<()>,
    fn_: fn(Query<T>, RouteContext<()>) -> U,
) -> CfResult<Response>
where
    T: for<'a> Deserialize<'a> + Debug + for<'a> serde::Deserialize<'a>,
    for<'a> U: Future<Output = CfResult<Response>> + 'a,
{
    let wrapper = Wrapper::new(
        Query::from_method(method, &req, &ctx),
        req,
        ctx,
        FnType::WithQuery(fn_),
    );
    wrapper.res().await.res()
}

pub async fn _private_wrap_with_req<T, U>(
    method: Method,
    req: Request,
    ctx: RouteContext<()>,
    fn_: fn(Query<T>, Request, RouteContext<()>) -> U,
) -> CfResult<Response>
where
    T: for<'a> Deserialize<'a> + Debug + for<'a> serde::Deserialize<'a>,
    for<'a> U: Future<Output = CfResult<Response>> + 'a,
{
    let wrapper = Wrapper::new(
        Query::from_method(method, &req, &ctx),
        req,
        ctx,
        FnType::WithReq(fn_),
    );

    wrapper.res().await.res()
}

// Ok(query_) => match fn_(query_, ctx).await {
//     Ok(res) => Ok(res),
//     Err(err) => ErrorJson::from(err).into(),
// },
// Err(err

// pub fn wrap_(
//     method: Method,
//     req: Request,
//     ctx: RouteContext<()>,
//     fn_: fn(Request, RouteContext<()>) -> CfResult<Response>,
// ) -> CfResult<Response> {
//     match fn_(req, ctx) {
//         Ok(query_) => Ok(query_),
//         Err(err) => ErrorJson::from(err).into(),
//     }
// }

// pub fn wrap_with_query_<T, U>(
//     method: Method,
//     req: Request,
//     ctx: RouteContext<()>,
//     fn_: fn(Query<T>, RouteContext<()>) -> U,
// ) -> CfResult<Response>
// where
//     T: for<'a> Deserialize<'a> + Debug + for<'a> serde::Deserialize<'a>,
// {
//     let wrapper = Wrapper::new(
//         Query::from_method(method, &req, &ctx),
//         req,
//         ctx,
//         FnType::WithQuery(fn_),
//     );
//     wrapper.fn_()
// }

// pub fn wrap_with_req_<T, U>(
//     method: Method,
//     req: Request,
//     ctx: RouteContext<()>,
//     fn_: fn(Query<T>, Request, RouteContext<()>) -> U,
// ) -> CfResult<Response>
// where
//     T: for<'a> Deserialize<'a> + Debug + for<'a> serde::Deserialize<'a>,
// {
//     let wrapper = Wrapper::new(
//         Query::from_method(method, &req, &ctx),
//         req,
//         ctx,
//         FnType::WithReq(fn_),
//     );

//     wrapper.res().res()
// }
