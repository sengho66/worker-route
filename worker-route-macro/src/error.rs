use proc_macro2::Span;

#[derive(Debug)]
pub struct Error(syn::Error);

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Self(value)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy)]
pub enum ErrorSpan {
    DuplicateMethod,
    InvalidFormat,
    InvalidMethod,
    InvalidMethods,
    Method,
}

impl ErrorSpan {
    fn message(self, ident: Option<&str>) -> String {
        match self {
            Self::DuplicateMethod =>  "duplicate method found".into(),
            Self::InvalidFormat => format!(r#"expected string literal, found {}"#, ident.unwrap()),
            Self::InvalidMethod => format!(r#"expected "get, head, delete, put, patch, post", found {}"#, ident.unwrap()),
            Self::InvalidMethods => format!(r#"expected "all, get, head, delete, put, patch, post", found {}"#, ident.unwrap()),
            Self::Method =>  format!("method's attribute is not supported when using {} attribute, use route instead or remove method", ident.unwrap()),
    }
    }
}

#[allow(clippy::module_name_repetitions)]
pub fn to_error<T: Into<String>>(e_span: ErrorSpan, ident: Option<T>, span: Span) -> syn::Error {
    let ident = ident.unwrap().into();
    syn::Error::new(span, e_span.message(Some(&ident)))
}
