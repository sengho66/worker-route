use crate::{
    http::HttpRequest,
    http::ResponseError,
    http::{InternalResponder, Responder},
    Query,
};
use futures::Future;
use serde::de::DeserializeOwned;
use worker::{Cors, Request, Response, RouteContext};

pub fn responder<T, E>(
    cors: Option<&Cors>,
    req: HttpRequest,
    res: Result<T, E>,
) -> worker::Result<Response>
where
    T: Responder,
    E: ResponseError,
{
    res.res(req, cors)
}

type WithQuery<D, T, U> = fn(Query<T>, RouteContext<D>) -> U;
type WithReq<D, T, U> = fn(Query<T>, Request, RouteContext<D>) -> U;

#[derive(Copy, Clone)]
pub enum FnType<D, T, U> {
    WithQuery(WithQuery<D, T, U>),
    WithReq(WithReq<D, T, U>),
}

#[allow(clippy::future_not_send)]
pub async fn respond_async<T: Responder, E: ResponseError>(
    req: HttpRequest,
    res: Result<impl Future<Output = Result<T, E>>, Box<dyn ResponseError>>,
    cors: Option<&Cors>,
) -> worker::Result<Response> {
    match res {
        Ok(res) => res.await.res(req, cors),
        Err(err) => Ok(err.error_response(req).into_response(cors).into_res()),
    }
}

trait Respond {
    fn res(self, req: HttpRequest, cors: Option<&Cors>) -> worker::Result<Response>;
}

impl<T, E> Respond for Result<T, E>
where
    T: Responder,
    E: ResponseError,
{
    fn res(self, req: HttpRequest, cors: Option<&Cors>) -> worker::Result<Response> {
        match self {
            Ok(res_) => Ok(res_.to_response(req).into_response(cors).into_res()),
            Err(err) => Ok(err.error_response(req).into_response(cors).into_res()),
        }
    }
}

macro_rules! impl_wrap_query {
    ($wrap:ident, $query:ident, $ctx:ident, $req:ident) => {
        match $wrap {
            Self::WithQuery(fn_) => fn_($query, $ctx),
            Self::WithReq(fn_) => fn_($query, $req, $ctx),
        }
    };

    ($ctx:ident, $req:ident) => {
        Query::_internal_query::<RouteContext<D>>($req.url(), &$ctx)
    };
}

impl<D, T, U, E> FnType<D, T, Result<U, E>>
where
    T: DeserializeOwned,
    U: Responder,
    E: ResponseError,
{
    pub fn wrap(
        wrap_: &Self,
        req: Request,
        ctx: RouteContext<D>,
        cors: Option<&Cors>,
    ) -> worker::Result<Response> {
        let http = HttpRequest::from(&req);
        match impl_wrap_query!(ctx, req) {
            Ok(query) => impl_wrap_query!(wrap_, query, ctx, req).res(http, cors),
            Err(err) => Ok(err.error_response(http).into_response(cors).into_res()),
        }
    }
}

impl<D, Q: DeserializeOwned, U> FnType<D, Q, U> {
    pub fn wrap_async<T, E>(
        wrap_: &Self,
        req: Request,
        ctx: RouteContext<D>,
    ) -> Result<U, Box<dyn ResponseError>>
    where
        U: Future<Output = Result<T, E>>,
        T: Responder,
        E: ResponseError,
    {
        match impl_wrap_query!(ctx, req) {
            Ok(query) => Ok(impl_wrap_query!(wrap_, query, ctx, req)),
            Err(err) => Err(Box::new(err)),
        }
    }
}
