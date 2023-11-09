use crate::wrapper::Wrapper;

use proc_macro2::{Ident, TokenStream};
use quote::ToTokens;
use syn::{
    parse2,
    punctuated::Punctuated,
    token::{Async, Comma},
    FnArg, ItemFn, ReturnType, Stmt, TypePath, Visibility,
};

pub struct FnWrapper<'a> {
    pub args: &'a Punctuated<FnArg, Comma>,
    pub asyncness: Option<Async>,
    pub data: TokenStream,
    pub name: &'a Ident,
    pub path: &'a str,
    pub ret: &'a ReturnType,
    pub route_context: TokenStream,
    pub stmts: &'a [Stmt],
    pub vis: &'a Visibility,
    pub wrapper: Result<TokenStream, TokenStream>,
}

impl<'a> FnWrapper<'a> {
    pub fn new(item: &'a ItemFn, path: &'a str, cors: &Option<TokenStream>) -> Self {
        let ItemFn {
            sig, block, vis, ..
        } = item;
        let stmts = &block.stmts;
        let name = &sig.ident;
        let args = &sig.inputs;
        let binding = args.last();
        let ctx = binding.unwrap();
        let is_query = args.first().unwrap();
        let query = is_query.to_token_stream();
        let asyncness = sig.asyncness;
        let ret = &sig.output;
        let wrapper = Wrapper::new(query.to_string(), args.len());
        let (route_context, data) = get_generic(ctx);

        let wrapper = wrapper.to_token(asyncness.is_some(), name, args, cors);

        Self {
            args,
            asyncness,
            data,
            name,
            path,
            ret,
            route_context,
            stmts,
            vis,
            wrapper,
        }
    }
}

// given an pub fn
// pub fn foo(req: Query<FooQuery>, ctx: RouteContext<CtxData>) -> Result<Response>
// get_generic() is used to extract RouteContext<CtxData>
// returning the parent and the child
fn get_generic(ctx: &FnArg) -> (TokenStream, TokenStream) {
    match ctx {
        FnArg::Typed(ty) => {
            let parent = ty.ty.to_token_stream();
            let parent = parse2::<TypePath>(parent).unwrap();
            let child = parent.path.segments[0].arguments.to_token_stream();
            let child = parse2::<syn::AngleBracketedGenericArguments>(child).unwrap();
            let child = child.args.last().unwrap().to_token_stream();

            (parent.to_token_stream(), child)
        }
        // don't worry about it, you only get this in your development environment
        // the correct arguments sequence are
        // (Request, RouteContext<D>)
        // or
        // (Query<T>, RouteContext<D>)
        // or
        // (Query<T>, Request, RouteContext<D>)
        // the sequence of the above must be in the correct order
        FnArg::Receiver(_) => panic!(),
    }
}
