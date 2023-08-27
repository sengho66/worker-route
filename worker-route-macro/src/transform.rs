use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, ToTokens};
use syn::{
    punctuated::Punctuated,
    token::{Async, Comma},
    FnArg, ItemFn, ReturnType, Stmt, Visibility,
};

pub struct FnWrapper {
    pub args: Punctuated<FnArg, Comma>,
    pub asyncness: Option<Async>,
    pub internal: Ident,
    pub is_async: bool,
    pub name: Ident,
    pub origin: Ident,
    pub path: String,
    pub ret: ReturnType,
    pub stmts: Vec<Stmt>,
    pub vis: Visibility,
    pub wrapper: TokenStream,
}

impl FnWrapper {
    pub fn new(item: ItemFn, path: String) -> Self {
        let ItemFn {
            sig, block, vis, ..
        } = item;
        let stmts = block.stmts;
        let origin = sig.ident;
        let name = format_ident!("internal_{}", origin.to_string());
        let internal = format_ident!("{}_", name.to_string());
        let args = sig.inputs;
        let is_query = args.first().unwrap();
        let query = is_query.to_token_stream();
        let asyncness = sig.asyncness;
        let is_async = asyncness.is_some();
        let wrapper = wrapper(&query.to_string(), args.len());
        let ret = sig.output;
        let path = path;

        Self {
            stmts,
            origin,
            wrapper,
            vis,
            name,
            asyncness,
            internal,
            ret,
            path,
            is_async,
            args,
        }
    }
}

fn wrapper(query: &str, len: usize) -> TokenStream {
    match len.cmp(&2) {
        // TODO
        // need a proper error message
        std::cmp::Ordering::Less => panic!(),
        // fn my_fn(req: Request, ctx: RouteContext<()>);
        // or
        // fn my_fn(req: Query<T>, ctx: RouteContext<()>);
        std::cmp::Ordering::Equal => match query.contains("Query") {
            true => quote::quote!(::worker_route::_private_wrap_with_query),
            _ => quote::quote!(::worker_route::_private_wrap),
        },
        // all 3
        // fn my_fn(req: Query<T>, _:Request, ctx: RouteContext<()>);
        std::cmp::Ordering::Greater => quote::quote!(::worker_route::_private_wrap_with_req),
    }
}
