use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{parse_quote, FnArg};

pub struct Wrapper {
    query__: String,
    len: usize,
}

impl Wrapper {
    pub const fn new(query__: String, len: usize) -> Self {
        Self { query__, len }
    }

    /*
     * len is extracted from the given fn's args.
     * Length:
     *  2 - (Default):
     *      fn my_fn(req__: Request, ctx__: RouteContext<()>)
     *
     *  2 - (With query__ without request):
     *      fn my_fn(query__: Query<T>, ctx__: RouteContext<()>)
     *
     *  3 - (With query__ and request):
     *      fn my_fn(query__: Query<T>, req__: Request, ctx__: RouteContext<()>)
     *
     * Anything below 2
     *
     * or
     *
     * the parameters do not follow the above's sequence
     * eg:
     *
     * Correct parameters order:
     * fn my_fn(query__: Query<T>, req__: Request, ctx__: RouteContext<()>)
     *
     * Incorrect parameters order:
     * fn my_fn(req__: Request, query__: Query<T>, ctx__: RouteContext<()>)
     *
     * will not compile, it will result a proc-macro panic.
     *
     * TODO:
     * Allow random parameters' sequence by refactoring this fn,
     * contribution is very much welcomed!
     *
     */
    #[allow(clippy::wrong_self_convention)]
    pub fn to_token(
        self,
        is_async: bool,
        name: &Ident,
        args: &Punctuated<FnArg, Comma>,
        cors: &Option<TokenStream>,
    ) -> Result<TokenStream, TokenStream> {
        if (2..=3).contains(&self.len) {
            return Ok(if self.query__.contains("Query") {
                Self::wrap(is_async, name, args, cors)
            } else {
                Self::wrap_default(is_async, name, args, cors)
            });
        }

        Err(Self::to_error(args))
    }

    fn call_fn(name: &Ident, is_async: bool, args_: &[TokenStream]) -> TokenStream {
        if is_async {
            parse_quote!(#name(#(#args_)*).await)
        } else {
            parse_quote!(#name(#(#args_)*))
        }
    }

    fn wrap_default(
        is_async: bool,
        name: &Ident,
        args: &Punctuated<FnArg, Comma>,
        cors: &Option<TokenStream>,
    ) -> TokenStream {
        let (args_, _) = collect_args(true, args);
        let (var, c) = Self::get_cors(cors);

        let ret = Self::call_fn(name, is_async, &args_);
        parse_quote!(
            #var
            ::worker_route::__private::responder(#c, ::worker_route::http::HttpRequest::from(&req__), #ret)
        )
        // if let Some(cors) = cors {
        //     return parse_quote!(::worker_route::__private::responder(&#cors, #ret));
        // }

        // ret
    }

    fn to_response(
        is_async: bool,
        name: &Ident,
        var: &Option<TokenStream>,
        cors: &TokenStream,
        fn_: &TokenStream,
    ) -> TokenStream {
        if !is_async {
            return parse_quote! {
                #var
                ::worker_route::__private::FnType::wrap(&#fn_(#name), req__, ctx__, #cors)
            };
        }
        parse_quote! {
            #var
            ::worker_route::__private::respond_async(::worker_route::http::HttpRequest::from(&req__), ::worker_route::__private::FnType::wrap_async(&#fn_(#name), req__, ctx__), #cors).await
        }
    }

    fn get_cors(cors: &Option<TokenStream>) -> (Option<TokenStream>, TokenStream) {
        cors.as_ref().map_or_else(
            || (None, quote!(None)),
            |cors| {
                (
                    Some::<TokenStream>(parse_quote! {
                    let __cors = &#cors;
                        }),
                    quote!(Some(__cors)),
                )
            },
        )
    }

    fn wrap(
        is_async: bool,
        name: &Ident,
        args: &Punctuated<FnArg, Comma>,
        cors: &Option<TokenStream>,
    ) -> TokenStream {
        let (var, c) = Self::get_cors(cors);
        let (_, fn_) = collect_args(false, args);

        Self::to_response(is_async, name, &var, &c, &fn_)
    }

    fn to_error(args: &Punctuated<FnArg, Comma>) -> TokenStream {
        let error = syn::Error::new(args.last().unwrap().span(), "invalid token");
        quote_spanned! {
            error.span() => compile_error!("invalid arguments");
        }
    }
}

pub fn get_param(is_default: bool, i: usize, len: usize) -> TokenStream {
    if is_default {
        if i < 2 {
            quote!(req__)
        } else {
            quote!(ctx__)
        }
    } else if i == 3 && len == 3 || i == 2 && len == 2 {
        quote!(ctx__)
    } else if i == 1 {
        quote!(query__)
    } else {
        quote!(req__)
    }
}

pub fn collect_args(
    is_default: bool,
    args: &Punctuated<FnArg, Comma>,
) -> (Vec<TokenStream>, TokenStream) {
    let len = args.len();
    let mut map_ = Vec::new();
    for (i, v) in args.iter().enumerate() {
        if let FnArg::Typed(_) = v {
            let ident_ = get_param(is_default, i + 1, len);
            map_.push(parse_quote!(#ident_,));
        }
    }
    assert_eq!(map_.len(), len);
    (
        map_,
        if len < 3 {
            quote!(::worker_route::__private::FnType::WithQuery)
        } else {
            quote!(::worker_route::__private::FnType::WithReq)
        },
    )
}
