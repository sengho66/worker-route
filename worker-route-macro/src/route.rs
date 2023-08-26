use std::fmt::Debug;

use crate::expand::expand;

use proc_macro::TokenStream as TokenStream_;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

#[allow(unused)]
#[derive(Clone, PartialEq, Hash, Eq)]
pub enum Method {
    Head = 0,
    Delete,
    Get,
    Options,
    Patch,
    Post,
    Put,
}

impl Method {
    pub fn to_token(&self) -> TokenStream {
        match self {
            Method::Delete => quote!(::worker::Method::Delete),
            Method::Get => quote!(::worker::Method::Get),
            Method::Head => quote!(::worker::Method::Head),
            Method::Options => quote!(::worker::Method::Options),
            Method::Patch => quote!(::worker::Method::Patch),
            Method::Post => quote!(::worker::Method::Post),
            Method::Put => quote!(::worker::Method::Put),
        }
    }

    pub fn to_compile_error(&self, error: syn::Error) -> TokenStream {
        match self {
            Method::Head => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[head("/path")]"#);
            },
            Method::Delete => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[delete("/path")]"#);
            },
            Method::Get => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[get("/path")]"#);
            },
            Method::Options => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[options("/path")]"#);
            },
            Method::Patch => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[patch("/path")]"#);
            },
            Method::Post => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[post("/path")]"#);
            },
            Method::Put => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[put("/path")]"#);
            },
        }
    }
}

pub fn new_route(attrs: TokenStream_, items: TokenStream_, method: Method) -> TokenStream_ {
    expand(attrs.into(), items.into(), method).into()
}

impl Debug for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = match self {
            Method::Head => "head",
            Method::Delete => "delete",
            Method::Get => "get",
            Method::Options => "options",
            Method::Patch => "patch",
            Method::Post => "post",
            Method::Put => "put",
        };

        f.write_str(v)
    }
}
