use core::fmt::{Debug, Display};

use crate::{
    error::{to_error, ErrorSpan},
    expand::expand,
    method::{Method, SUPPORTED_METHODS},
};

use proc_macro::TokenStream as TokenStream_;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse, parse_macro_input, parse_quote, spanned::Spanned, token::Async, ExprArray, Lit, LitStr,
    Token,
};

pub fn gen_router(
    asyncness: Option<Async>,
    cors: &Option<TokenStream>,
    path: &str,
    method: &str,
    wrap: bool,
    is_last: Option<(usize, usize)>,
) -> TokenStream {
    let method_ = method;
    let method: TokenStream = Method::new(&method.to_uppercase()).to_token().into();
    let register = if asyncness.is_some() {
        quote!(register_async)
    } else {
        quote!(register)
    };
    let mut chain = None;
    let args: TokenStream = parse_quote!(
        #path,
        #method,
        __handler
    );
    let mut call: TokenStream = parse_quote!(router__.#register);

    if let Some((i, len)) = is_last {
        if i + 1 != len {
            chain = Some(quote!(.));
        }

        if i > 0 {
            call = parse_quote!(#register);
            // args = parse_quote!(
            //     #path,
            //     #method,
            //     __handler
            // );
        }

        if (method_ == "options" || i + 1 == len) && wrap && cors.is_some() {
            let (options, move_) = if asyncness.is_some() {
                (quote!(options_async), Some(quote!(async move)))
            } else {
                (quote!(options), None)
            };

            let register_ = parse_quote!(#options(#path, |req__, _| #move_ {
                ::worker::Response::empty()?.with_cors(&#cors)
            }));

            if len == 7 {
                return register_;
            }

            return parse_quote! {
                #call(#args).#register_
            };
        }
    }

    parse_quote! {
        #call(#args)#chain
    }
}

#[derive(Default)]
pub struct Route {
    pub path: Option<String>,
    pub cors: Option<Ident>,
    pub lazy_cors: Option<Ident>,
    pub methods: Vec<LitStr>,
    pub wrap: bool,
    pub is_single: bool,
}

impl Debug for Route {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let cors = if self.cors.is_none() {
            quote!(None)
        } else {
            self.cors.as_ref().unwrap().to_token_stream()
        };
        let lazy_cors = if self.lazy_cors.is_none() {
            quote!(None)
        } else {
            self.lazy_cors.as_ref().unwrap().to_token_stream()
        };
        if !self.is_single {
            let methods = self
                .methods
                .iter()
                .map(quote::ToTokens::to_token_stream)
                .collect::<Vec<_>>();

            return f.write_str(&format!(
                "route({:?}, method: [{:?}], cors: {}, lazy_cors: {}, wrap: {})",
                self.path.as_ref().unwrap(),
                methods,
                cors,
                lazy_cors,
                self.wrap
            ));
        }
        let attr = self.methods[0].value().to_lowercase();

        f.write_str(&format!(
            "{attr}({:?}, cors: {:?}, lazy_cors: {:?}, wrap: {})",
            self.path.as_ref().unwrap(),
            cors,
            lazy_cors,
            self.wrap
        ))
    }
}

impl Display for Route {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let cors = if self.cors.is_none() {
            quote!(None)
        } else {
            self.cors.as_ref().unwrap().to_token_stream()
        };
        let lazy_cors = if self.lazy_cors.is_none() {
            quote!(None)
        } else {
            self.lazy_cors.as_ref().unwrap().to_token_stream()
        };

        if !self.is_single {
            let methods = self.methods.iter().map(quote::ToTokens::to_token_stream);
            return write!(
                f,
                "route({:?}, method: [{}], cors: {}, lazy_cors: {}, wrap: {})",
                self.path.as_ref().unwrap(),
                quote!(#(#methods)*),
                cors,
                lazy_cors,
                self.wrap
            );
        }
        let attr = self.methods[0].value().to_lowercase();

        write!(
            f,
            "{attr}({:?}, cors: {}, lazy_cors: {}, wrap: {})",
            self.path.as_ref().unwrap(),
            cors,
            lazy_cors,
            self.wrap
        )
    }
}

struct RouteArgs<T, const U: usize>(Route, Option<T>);

pub fn with_method<const T: usize>(attrs: TokenStream_, items: TokenStream_) -> TokenStream_ {
    let args = parse_macro_input!(attrs with RouteArgs::<Method, T>::parse);
    expand(items.into(), &args.0).into()
}

enum CorsVariant {
    Default,
    Lazy,
}

impl CorsVariant {
    fn to_variant(variant: &str) -> Self {
        match variant {
            "cors" => Self::Default,
            _ => Self::Lazy,
        }
    }
}

fn fill(route: &mut Route, input: parse::ParseStream) -> syn::Result<()> {
    if let Ok(ident) = input.parse::<Ident>() {
        let ident_ = ident.to_string();
        match ident_.as_str() {
            "method" => route.get_method(input, &ident)?,
            "cors" | "lazy_cors" => route.get_cors(&ident_, input)?,
            "wrap" => route.wrap(&ident)?,
            _ => {
                let error = syn::Error::new(
                    ident.span(),
                    format!(
                        r#"expected #[route("<path>", method, cors/lazy_cors)], found {}"#,
                        ident.span().source_text().unwrap()
                    ),
                );
                return Err(error);
            }
        }
        fill(route, input)?;
    }

    Ok(())
}

impl<T, const U: usize> RouteArgs<T, U> {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let mut route = Route::new(Method::to_method(U));

        match input.parse::<LitStr>() {
            Ok(path) => route.path = Some(path.value()),
            Err(e) => {
                return Err(input.error(format!(
                    r#"expected #[route("<path>")], found {}"#,
                    e.span().source_text().unwrap()
                )));
            }
        }
        if !route.is_single {
            match input.parse::<Token![,]>() {
                Ok(_) => fill(&mut route, input)?,
                Err(_) => {
                    return Err(syn::Error::new(
                        input.span(),
                        r#"required property "method" is missing"#,
                    ))
                }
            }
        } else if input.parse::<Token![,]>().is_ok() {
            fill(&mut route, input)?;
        }

        Ok(Self(route, None))
    }
}

impl Route {
    fn new(method: Option<Method>) -> Self {
        let mut self_ = Self::default();
        if let Some(method) = method {
            let method = syn::LitStr::new(&String::from(method), Span::call_site());
            self_.is_single = true;
            self_.methods.push(method);
        }

        self_
    }

    fn insert_method(&mut self, method: LitStr, expr: Span) -> syn::Result<()> {
        if method.value() == "all" {
            if !self.methods.is_empty() {
                if self.methods.len() == 7 {
                    return Err(to_error(
                        ErrorSpan::DuplicateMethod,
                        Some(&method.value()),
                        method.span(),
                    ));
                }
                return Err(to_error(
                    ErrorSpan::InvalidMethod,
                    Some(&method.value()),
                    method.span(),
                ));
            }
            self.methods.extend(Method::all());
        } else {
            if self.is_single {
                return Err(to_error(ErrorSpan::Method, expr.source_text(), expr));
            }
            if !SUPPORTED_METHODS.contains(&method.value().to_uppercase().as_str()) {
                return Err(to_error(
                    ErrorSpan::InvalidMethods,
                    Some(&method.value()),
                    method.span(),
                ));
            }

            if self.methods.iter().any(|v| v.value() == method.value()) {
                return Err(to_error(
                    ErrorSpan::DuplicateMethod,
                    Some(&method.value()),
                    method.span(),
                ));
            }
            self.methods.push(method);
        }

        Ok(())
    }

    fn wrap(&mut self, ident: &Ident) -> syn::Result<()> {
        if self.cors.is_none() && self.lazy_cors.is_none() {
            let error = syn::Error::new(
                ident.span(),
                r#"wrap cannot be used when "cors/lazy_cors" are ommitted"#,
            );
            return Err(error);
        }
        self.wrap = true;
        if self.wrap
            && self
                .methods
                .iter()
                .any(|v| v.value().to_lowercase() == "options")
            && self.methods.len() != 7
        {
            let error = syn::Error::new(
                ident.span(),
                "wrap cannot be used when options handler is present",
            );
            return Err(error);
        }
        Ok(())
    }

    fn get_method(&mut self, input: parse::ParseStream, ident: &Ident) -> syn::Result<()> {
        let _ = input.parse::<Token![=]>()?;

        if let Ok(o) = input.parse::<ExprArray>() {
            for i in o.elems {
                match i {
                    syn::Expr::Lit(lit) => {
                        if let Lit::Str(method) = lit.lit {
                            self.insert_method(method, ident.span())?;
                        }
                    }

                    _ => {
                        return Err(to_error(
                            ErrorSpan::InvalidFormat,
                            i.span().source_text(),
                            i.span(),
                        ));
                    }
                }
            }
        }

        if let Ok(method) = input.parse::<LitStr>() {
            self.insert_method(method, ident.span())?;
        }

        if input.parse::<Token![,]>().is_ok() {}

        Ok(())
    }

    fn get_cors(&mut self, ident_: &str, input: parse::ParseStream) -> syn::parse::Result<()> {
        let _ = input.parse::<Token![=]>()?;
        let cors = input.parse::<Ident>()?;

        match CorsVariant::to_variant(ident_) {
            CorsVariant::Default => {
                self.cors = Some(cors);
            }
            CorsVariant::Lazy => {
                self.lazy_cors = Some(cors);
            }
        }

        if input.parse::<Token![,]>().is_ok() {}

        Ok(())
    }

    pub fn cors(&self) -> Option<TokenStream> {
        match (&self.cors, &self.lazy_cors) {
            (None, Some(lazy_cors)) => Some(quote! {
                #lazy_cors
            }),
            (Some(cors), None) => Some(quote! {
                <#cors as ::worker_route::Wrap>::wrap(&req__)
            }),
            _ => None,
        }
    }
}
