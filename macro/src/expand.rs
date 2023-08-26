use proc_macro2::TokenStream;
use quote::quote;
use syn::parse2;

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

    let expanded = quote! {
        #[allow(clippy::unused_async)]
        #asyncness #vis fn #name(#args) #ret {
            #(#stmts)*
        }
        #[allow(clippy::unused_async)]
        #asyncness #vis fn #internal(req: ::worker::Request, ctx: ::worker::RouteContext<()>) #ret {
            #wrapper(#method, req, ctx, #name).await
        }
        #[allow(clippy::unused_async)]
        #vis fn #origin() -> ::worker_route::RouteHandler<
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

