use proc_macro::{Span, TokenStream};
use quote::{quote, quote_spanned};
use core::fmt::Debug;
use syn::LitStr;

#[allow(unused)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum Method {
    Default = 0,
    Delete = 1,
    Get = 2,
    Head = 3,
    Options = 4,
    Patch = 5,
    Post = 6,
    Put = 7,
}

pub const SUPPORTED_METHODS: &[&str] = &[
    "ALL", "GET", "DELETE", "HEAD", "PATCH", "POST", "PUT", "OPTIONS",
];

impl From<Method> for String {
    fn from(value: Method) -> Self {
        match value {
            Method::Default => Self::from("NONE"),
            Method::Delete => Self::from("DELETE"),
            Method::Get => Self::from("GET"),
            Method::Head => Self::from("HEAD"),
            Method::Options => Self::from("OPTIONS"),
            Method::Patch => Self::from("PATCH"),
            Method::Post => Self::from("POST"),
            Method::Put => Self::from("PUT"),
        }
    }
}

impl Method {
    pub const fn to_method(method: usize) -> Option<Self> {
        match method {
            1 => Some(Self::Delete),
            2 => Some(Self::Get),
            3 => Some(Self::Head),
            4 => Some(Self::Options),
            5 => Some(Self::Patch),
            6 => Some(Self::Post),
            7 => Some(Self::Put),
            _ => None,
        }
    }

    pub fn all() -> Vec<LitStr> {
        [
            LitStr::new("delete", Span::call_site().into()),
            LitStr::new("get", Span::call_site().into()),
            LitStr::new("head", Span::call_site().into()),
            LitStr::new("patch", Span::call_site().into()),
            LitStr::new("post", Span::call_site().into()),
            LitStr::new("put", Span::call_site().into()),
            LitStr::new("options", Span::call_site().into()),
        ]
        .into()
    }

    pub fn new(method: &str) -> Self {
        match method {
            "DELETE" => Self::Delete,
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "OPTIONS" => Self::Options,
            "PATCH" => Self::Patch,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            _ => todo!(),
        }
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn to_token(self) -> TokenStream {
        match self {
            Self::Delete => quote!(::worker::Method::Delete).into(),
            Self::Get => quote!(::worker::Method::Get).into(),
            Self::Head => quote!(::worker::Method::Head).into(),
            Self::Options => quote!(::worker::Method::Options).into(),
            Self::Patch => quote!(::worker::Method::Patch).into(),
            Self::Post => quote!(::worker::Method::Post).into(),
            Self::Put => quote!(::worker::Method::Put).into(),
            _ => todo!(),
        }
    }

    #[allow(clippy::match_wildcard_for_single_variants)]
    pub fn to_compile_error(self, error: &syn::Error) -> TokenStream {
        match self {
            Self::Delete => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[delete("/path")]"#);
            }
            .into(),
            Self::Get => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[get("/path")]"#);
            }
            .into(),
            Self::Head => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[head("/path")]"#);
            }
            .into(),
            Self::Options => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[options("/path")]"#);
            }
            .into(),
            Self::Patch => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[patch("/path")]"#);
            }
            .into(),
            Self::Post => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[post("/path")]"#);
            }
            .into(),
            Self::Put => quote_spanned! {
                error.span() =>
                compile_error!(r#"unexpected token, valid token is #[put("/path")]"#);
            }
            .into(),
            _ => todo!(),
        }
    }
}

impl Debug for Method {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v = match self {
            Self::Delete => "DELETE",
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Patch => "PATCH",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Options => "OPTIONS",
            Self::Default => "NONE",
        };

        f.write_str(v)
    }
}
