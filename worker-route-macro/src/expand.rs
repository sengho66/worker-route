use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse2, FnArg, TypePath};

use crate::error::Error;
use crate::route::Method;
use crate::transform::FnWrapper;

pub fn expand(attrs_: TokenStream, items: TokenStream, method: Method) -> TokenStream {
    let path = match get_path(attrs_) {
        Ok(p) => match p {
            syn::Lit::Str(s) => s.value(),
            _ => todo!(),
        },
        Err(e) => return method.to_compile_error(e.inner()),
    };

    let method = method.to_token();
    let fn_ = expand_fn(items).unwrap();
    let ctx = fn_.sig.inputs.last();
    let (parent, child) = get_generic(ctx.unwrap());
    let FnWrapper {
        stmts,
        origin,
        asyncness,
        name,
        internal,
        ret,
        path,
        args,
        wrapper,
        vis,
        is_async,
    } = FnWrapper::new(fn_, path);

    // struct CtxData;
    // struct FooQuery;
    // fn foo(req: Query<FooQuery>, ctx: RouteContext<CtxData>) -> Result<Response>;
    //
    // becomes
    // fn foo() -> RouteHandler<CtxData, impl Future<Output = Result<Response>>>;
    //
    // generated two additional functions
    // fn internal_foo_(req, Request, RouteContext<CtxData) -> Result<Response>;
    //
    // below is the original function renamed to internal_foo
    // fn internal_foo(req: Query<FooQuery>, ctx: RouteContext<CtxData>) -> Result<Response>;
    //
    // run cargo expand to view what the additional functions do

    let expanded = quote! {
        #[allow(clippy::unused_async)]
        #asyncness #vis fn #name(#args) #ret {
            #(#stmts)*
        }
        #[allow(clippy::unused_async)]
        #asyncness #vis fn #internal(req: ::worker::Request, ctx: #parent) #ret {
            #wrapper(#method, req, ctx, #name).await
        }
        #[allow(clippy::unused_async)]
        #vis fn #origin() -> ::worker_route::RouteHandler<#child,
        impl ::core::future::Future<Output = ::worker_route::CfResult<::worker::Response>>> {
            ::worker_route::RouteHandler::new(#internal, #path, #is_async, #method)
        }
    };

    expanded
}

pub fn get_path(attrs: TokenStream) -> Result<syn::Lit, Error> {
    Ok(parse2::<syn::Lit>(attrs)?)
}

pub fn expand_fn(items: TokenStream) -> Result<syn::ItemFn, Error> {
    Ok(parse2::<syn::ItemFn>(items)?)
}


// given an fn
// fn foo(req: Query<FooQuery>, ctx: RouteContext<CtxData>) -> Result<Response>
// get_generic() is used to extract RouteContext<CtxData>
// returning the parent and the child
fn get_generic(ctx: &FnArg) -> (TokenStream, TokenStream) {
    match ctx {
        FnArg::Typed(ty) => {
            let parent = ty.ty.to_token_stream();
            let parent = parse2::<TypePath>(parent.clone()).unwrap();
            let child = parent.path.segments[0].arguments.to_token_stream();
            let child = parse2::<syn::AngleBracketedGenericArguments>(child.clone()).unwrap();
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
        _ => panic!(),
    }
}
