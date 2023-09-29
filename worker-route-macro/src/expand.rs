use crate::error::Error;
use crate::route::{gen_router, Route};
use crate::transform::FnWrapper;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, parse_quote};

#[allow(clippy::module_name_repetitions)]
pub fn expand_fn(items: TokenStream) -> Result<syn::ItemFn, Error> {
    Ok(parse2::<syn::ItemFn>(items)?)
}

// Code is unreadable atm, will be refactored in the near future!
pub fn expand(items: TokenStream, route: &Route) -> TokenStream {
    let path = route.path.as_ref().unwrap();
    let fn_ = expand_fn(items).unwrap();
    let cors = route.cors();
    let FnWrapper {
        asyncness,
        path,
        name,
        ret,
        vis,
        stmts,
        args,
        wrapper,
        route_context,
        data,
    } = FnWrapper::new(&fn_, path, &cors);

    let attr = if asyncness.is_some() {
        Some(quote!(#[allow(clippy::unused_async)]))
    } else {
        None
    };
    let wrapper = match wrapper {
        Ok(w) => w,
        Err(e) => return e,
    };
    let attrs = &fn_.attrs;

    let routes = route.methods.iter().enumerate().map(|(i, v)| {
        gen_router(
            asyncness,
            &cors,
            path,
            &v.value(),
            route.wrap,
            Some((i, route.methods.len())),
        )
    });

    let expanded = parse_quote! {
        #(#attrs)*
        #[allow(non_camel_case_types, missing_docs)]
        pub struct #name;
        impl ::worker_route::__private::RouteFactory<#data> for #name {
            fn register(
                self,
                router__: ::worker::Router<'_, #data>
            ) -> ::worker::Router<'_, #data> {
                use ::worker_route::__private::AddHandler;
                pub #asyncness fn __handler(
                    req__: ::worker::Request,
                    ctx__: #route_context
                ) -> ::worker::Result<::worker::Response> {
                    #attr
                    #[allow(missing_docs)]
                    #vis #asyncness fn #name(#args) #ret {
                        #(#stmts)*
                    }
                    #wrapper
                }
                #(#routes)*
            }
        }
    };

    expanded
}
