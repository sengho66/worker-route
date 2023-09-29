use crate::error::Error;
use crate::error::ErrorCause;
use crate::route::Params;
use crate::utils::struct_fields;
use crate::utils::StructFields;

use core::fmt::Debug;
use core::fmt::Display;
use http::StatusCode;
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::ops::Deref;
use worker::Url;

/// Extract typed information with the supplied struct and deserialize it with [`worker::Url`](worker::Url).
///
/// To extract typed data from [`worker::Url`](worker::Url), `T` must implement
/// the [`DeserializeOwned`](serde::de::DeserializeOwned) trait.
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use worker::{Response, Result, RouteContext};
/// use worker_route::{get, Query};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct StructFoo {
///     foo: String,
/// }
///
/// #[get("/foo-struct")]
/// async fn struct_foo(req: Query<StructFoo>, _: RouteContext<()>) -> Result<Response> {
///     // works
///     let Foo { foo } = req.into_inner();
///     // rest code
/// }
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct TupleFoo(String);
///
/// #[get("/foo-tuple")]
/// async fn tuple_foo(req: Query<TupleFoo>, _: RouteContext<()>) -> Result<Response> {
///     // you won't even get here
///     let TupleFoo ( foo ) = req.into_inner();
///     // rest code
/// }
///
/// ```
///
/// # Notes
/// Request can be an ommited from the parameter too.
/// When ommitting either of them, the sequence must always be in the correct order.
///
/// The correct orders are:
/// - (`Request`, `RouteContext<D: Params>`)
/// - (`Query<T>`, `RouteContext<D: Params>`)
/// - (`Query<T>`, `Request`, `RouteContext<D: Params>`)
///
/// ```
/// use serde::{Deserialize, Serialize};
/// use worker::{Response, Request, Result, RouteContext};
/// use worker_route::{get, Query};
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Foo {
///     foo: String,
/// }
///
/// #[get("/foo-query")]
/// async fn without_req(req: Query<Foo>, _: RouteContext<()>) -> Result<Response> {
///     // rest code
///     Response::empty()
/// }
///
/// #[get("/foo-with-request")]
/// async fn with_request(req: Query<Foo>, _: Request, _: RouteContext<()>) -> Result<Response> {
///     // rest code
///     Response::empty()
/// }
/// ```
///
#[derive(Debug, Clone)]
pub struct Query<T>(T);

impl<T> Query<T> {
    #[allow(clippy::missing_const_for_fn)]
    /// Acess the owned `T`
    pub fn into_inner(self) -> T {
        self.0
    }
}

fn paths<D: Params>(field: &str, ctx: &D) -> Option<String> {
    ctx.param_(field)
        .map(|p| format!("{}={}", field.trim(), p.trim()))
}

fn map_param(v: (Cow<'_, str>, Cow<'_, str>)) -> (Box<str>, Box<str>) {
    (v.0.into(), v.1.into())
}

struct QueryFields(Vec<(Box<str>, Box<str>)>);

enum QueryError {
    Duplicate(String),
    Unrecognized(String),
    Unexpected((String, String)),
}

impl QueryError {
    fn to_error(&self) -> Result<(), Error> {
        Err(Error::new(
            self.to_string(),
            StatusCode::BAD_REQUEST,
            ErrorCause::Query,
        ))
    }
}

impl Display for QueryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let str_ = match self {
            QueryError::Duplicate(duplicate) => {
                format!("duplicate query parameters found: `{duplicate}`")
            }
            QueryError::Unrecognized(unrecognized) => {
                format!("unexpected query parameters found: `{unrecognized}`")
            }
            QueryError::Unexpected((key, unexpected)) => {
                format!("expected `{key}`, found `{unexpected}`")
            }
        };

        f.write_str(&str_)
    }
}

trait FilterQuery<T> {
    type Output;
    fn filter_query(self, field: T) -> Self::Output;
}

impl FilterQuery<&str> for &[(Box<str>, Box<str>)] {
    type Output = bool;

    fn filter_query(self, field: &str) -> Self::Output {
        self.iter().filter(|pair| *pair.0 == *field).count() > 1
    }
}

impl FilterQuery<&str> for &[&str] {
    type Output = Option<Box<str>>;

    fn filter_query(self, field: &str) -> Self::Output {
        (!self.contains(&field)).then_some(field.into())
    }
}

impl FilterQuery<char> for &&String {
    type Output = bool;

    fn filter_query(self, field: char) -> Self::Output {
        self.contains(field)
    }
}

impl Debug for QueryFields {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let fields = self
            .0
            .iter()
            .map(|v| format!("key: {}, value: {}", v.0, v.1))
            .collect::<Vec<String>>()
            .join(", ");
        let fields = format!("QueryFields {{ pairs: [{fields}] }}");

        f.write_str(&fields)
    }
}

impl QueryFields {
    fn inserted(&mut self, iter: impl Iterator<Item = String>) -> Vec<usize> {
        let mut found = Vec::new();
        for (index, i) in iter.enumerate() {
            if let Some(current) = i.find('=') {
                self.0
                    .insert(index, (i[..current].into(), i[(current + 1)..].into()));
                found.push(index);
            }
        }
        found
    }

    fn new(url: &Url) -> Self {
        Self(url.query_pairs().map(map_param).collect())
    }

    fn ordered_fields(mut self, paths: &mut Vec<String>, fields: &[&str]) -> Result<(), Error> {
        self.inserted(paths.iter().filter(|v| v.filter_query('=')).cloned())
            .iter()
            .for_each(|i| {
                paths.remove(*i);
            });

        self.collect_duplicated(fields)?;
        self.collect_unrecognized(fields)?;

        if paths.len() != fields.len() {
            for (index, key) in fields.iter().enumerate() {
                if let Some(query) = &self.0.get(index) {
                    // this seems like never get reached, someone test this please
                    if key != &&*query.0 {
                        use QueryError::Unexpected;
                        Unexpected(((*key).to_owned(), query.0.to_string())).to_error()?;
                    }
                    if !query.1.is_empty() {
                        paths.push(format!("{key}={}", query.1));
                    }
                }
            }
        }

        Ok(())
    }

    fn collect_duplicated(&mut self, fields: &[&str]) -> Result<(), Error> {
        let duplicated = fields
            .iter()
            .filter(|v| FilterQuery::filter_query(self.0.as_slice(), **v))
            .copied()
            .collect::<Box<_>>();

        if !duplicated.is_empty() {
            QueryError::Duplicate(duplicated.join(", ")).to_error()?;
        }

        Ok(())
    }

    fn collect_unrecognized(&mut self, fields: &[&str]) -> Result<(), Error> {
        let unrecognized = self
            .0
            .iter()
            .filter_map(
                |v| FilterQuery::filter_query(fields, (*v.0).into()), // (!fields.contains(&&*v.0)).then_some(v.0.clone())
            )
            .collect::<Box<_>>();

        if !unrecognized.is_empty() {
            QueryError::Unrecognized(unrecognized.join(", ")).to_error()?;
        }

        Ok(())
    }
}

fn unordered_fields(paths: &mut Vec<String>, fields: &[&str], query: &str) {
    // query may have all the params
    // but they also might not be in the same order as the supplied struct
    //
    // eg:
    // supplied struct: struct Person { name: String, age: usize }
    // query: "age=20&name=Nick"
    // the above will result a panic due to wrong sequence of queries
    // when deserializing it with serde_qs
    //
    // the below re-arranges the sequence to construct a new query string
    if paths.len() != fields.len() {
        for i in fields {
            if let Some(start) = query.find(i) {
                let query_ = &query[start..];
                let end = query_.find('&').unwrap_or(query_.len());
                if end <= query_.len() {
                    let param = &query_[0..end];

                    // ensures the field is not empty
                    // eg:
                    // invalid: "name=Nick&age"
                    // invalid: "name=Nick&age="
                    // valid: "name=Nick&age=2"
                    if param.contains('=') {
                        paths.push(param.to_owned());
                    }
                }
            }
        }
    }
}

impl<T: DeserializeOwned> Query<T> {
    fn collect_paths<D: Params>(fields: StructFields, ctx: &D) -> Vec<String> {
        fields.iter().filter_map(|v| paths(v, ctx)).collect()
    }

    fn new<D: Params>(url: &Url, ctx: &D, strict: bool) -> Result<Self, Error> {
        let fields = struct_fields::<T>()?;
        // get fields from path first and merge them later on
        // "/my_path/:name/:age"
        let mut paths = Self::collect_paths(fields, ctx);
        // if the given route is "/my_path/{some_params}" then paths.len() should be empty
        // or if the given route is "/my_path/:name/{some_optional_params}"
        // then we try getting them from the url instead
        if let Some(query) = url.query() {
            if strict {
                // deny any possible unexpected/duplicate parameters
                // eg: /path/name + optional parameters page/sort
                // /path/Foo?page=10&sort=true -> Ok
                // /path/Foo?sort=true&page=10 -> Err
                QueryFields::new(url).ordered_fields(&mut paths, fields)?;
            } else {
                // ignore any unexpected/duplicate parameters
                // eg: /path/name + optional parameters page/sort
                // /path/Foo?page=10&sort=true&random_param=hello -> Ok
                // /path/Foo?random_param=hello&sort=true&page=10 -> Ok too
                unordered_fields(&mut paths, fields, query);
            }
        }

        let queries = paths.join("&");
        Ok(Self(serde_qs::from_str::<T>(&queries)?))
    }

    /// Deserialize the given `T` from the URL query string.
    ///
    /// ```
    /// use serde::{Deserialize, Serialize};
    /// use worker::{console_log, Request, Response, Result, RouteContext};
    /// use worker_route::{get, Query};
    ///
    /// #[derive(Debug, Deserialize, Serialize)]
    /// struct Person {
    ///     name: String,
    ///     age: usize,
    /// }
    ///
    /// #[get("/persons/:name/:age")]
    /// async fn person(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    ///     let person = Query::<Person>::from_query_path(&req.url().unwrap(), &ctx, true);
    ///     let Person { name, age } = person.unwrap().into_inner();
    ///     console_log!("name: {name}, age: {age}");
    ///     Response::empty()
    /// }
    ///
    /// ```
    ///
    /// # Errors
    /// Currently only regular structs are supported.
    /// Errors are returned if the given `T` is not a regular struct (eg: tuple, unit).
    ///
    pub fn from_query_path<D: Params>(url: &Url, ctx: &D, strict: bool) -> Result<Self, Error> {
        Self::new(url, ctx, strict)
    }

    #[doc(hidden)]
    pub fn _internal_query<D: Params>(url: worker::Result<Url>, ctx: &D) -> Result<Self, Error> {
        match url {
            Ok(url) => Query::<T>::from_query_path(&url, ctx, true),
            Err(e) => Err(crate::error::Error::new(
                e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCause::Query,
            )),
        }
    }
}

impl<T: Display> Display for Query<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Query<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Query<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> std::ops::DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use crate::{route::Params, Query};

    use serde::Deserialize;
    use std::collections::HashMap;
    use worker::Url;

    type CustomParam = HashMap<String, String>;
    impl Params for CustomParam {
        fn param_(&self, key: &str) -> Option<&String> {
            self.get(key)
        }
    }

    #[allow(unused)]
    #[derive(Deserialize, Debug)]
    struct OptionalProfile {
        page: Option<usize>,
        sort_by: Option<String>,
        order_by: Option<String>,
    }

    #[test]
    fn no_missing_fields() {
        let url =
            Url::parse("http://127.0.0.1:8787/profile?page=1&sort_by=email&order_by=desc").unwrap();
        let params = CustomParam::new();
        let query = Query::<OptionalProfile>::from_query_path(&url, &params, true).unwrap();

        assert_eq!(query.0.order_by, Some(String::from("desc")));
        assert_eq!(query.0.sort_by, Some(String::from("email")));
        assert_eq!(query.0.page, Some(1));
    }

    #[test]
    fn optional_fields() {
        let url = Url::parse("http://127.0.0.1:8787/profile?page=1").unwrap();
        let params = CustomParam::new();
        let query = Query::<OptionalProfile>::from_query_path(&url, &params, true).unwrap();

        assert_eq!(query.0.order_by, None);
        assert_eq!(query.0.sort_by, None);
        assert_eq!(query.0.page, Some(1));
    }

    #[allow(unused)]
    #[derive(Deserialize, Debug)]
    struct Profile {
        page: usize,
        sort_by: Option<String>,
        order_by: Option<String>,
    }

    #[test]
    fn required_fields_ok() {
        let url = Url::parse("http://127.0.0.1:8787/profile?page=1").unwrap();
        let params = CustomParam::new();
        let query = Query::<Profile>::from_query_path(&url, &params, true).unwrap();

        assert_eq!(query.0.order_by, None);
        assert_eq!(query.0.sort_by, None);
        assert_eq!(query.0.page, 1);
    }

    #[test]
    fn required_fields_err() {
        let url = Url::parse("http://127.0.0.1:8787/profile").unwrap();
        let params = CustomParam::new();
        let query = Query::<Profile>::from_query_path(&url, &params, true);

        assert!(query.is_err());
    }

    #[test]
    fn strict_ordered_queries_ok() {
        let url =
            Url::parse("http://127.0.0.1:8787/profile?sort_by=last_name&order_by=asc").unwrap();
        let mut params = CustomParam::new();
        params.insert("page".into(), "10".into());
        let query = Query::<Profile>::from_query_path(&url, &params, true).unwrap();
        let query = query.into_inner();
        assert_eq!(query.page, 10);
        assert_eq!(query.sort_by.unwrap(), String::from("last_name"));
        assert_eq!(query.order_by.unwrap(), String::from("asc"));
    }

    #[test]
    fn strict_ordered_queries_err() {
        let url =
            Url::parse("http://127.0.0.1:8787/profile?order_by=last_name&sort_by=asc").unwrap();
        let mut params = CustomParam::new();
        params.insert("page".into(), "10".into());

        let query = Query::<Profile>::from_query_path(&url, &params, true);
        assert!(query.is_err());
    }

    #[allow(unused)]
    #[derive(Deserialize, Debug)]
    struct ManyFields {
        age: i64,
        date: String,
        email: String,
        first_name: String,
        gender: String,
        last_name: String,
    }

    #[test]
    fn many_fields_ok() {
        let url = Url::parse("http://127.0.0.1:8787/profile?age=20&date=randomdate&first_name=Foo&gender=male&last_name=Bar&email=elon_musk@gmail.com").unwrap();
        let age = 20;
        let date = "randomdate";
        let email = "elon_musk@gmail.com";
        let first_name = "Foo";
        let gender = "male";
        let last_name = "Bar";
        let params = CustomParam::new();
        let query = Query::<ManyFields>::from_query_path(&url, &params, false).unwrap();

        let fields = query.into_inner();
        assert_eq!(fields.age, age);
        assert_eq!(fields.date, date);
        assert_eq!(fields.email, email);
        assert_eq!(fields.first_name, first_name);
        assert_eq!(fields.gender, gender);
        assert_eq!(fields.last_name, last_name);
    }

    #[test]
    fn many_fields_err() {
        // ommit date
        let url = Url::parse("http://127.0.0.1:8787/profile?age=20&first_name=Foo&gender=male&last_name=Bar&email=elon_musk@gmail.com").unwrap();
        let params = CustomParam::new();
        let query = Query::<ManyFields>::from_query_path(&url, &params, true);

        assert!(query.is_err());
    }

    #[test]
    fn path_with_query() {
        let url = Url::parse("http://127.0.0.1:8787/profile?age=20&date=randomdate&gender=male&last_name=Bar&email=elon_musk@gmail.com").unwrap();
        let mut params = CustomParam::new();
        params.insert("first_name".into(), "Foo".into());

        let age = 20;
        let date = "randomdate";
        let email = "elon_musk@gmail.com";
        let first_name = "Foo";
        let gender = "male";
        let last_name = "Bar";
        let query = Query::<ManyFields>::from_query_path(&url, &params, false).unwrap();
        let fields = query.into_inner();

        assert_eq!(fields.age, age);
        assert_eq!(fields.date, date);
        assert_eq!(fields.email, email);
        assert_eq!(fields.first_name, first_name);
        assert_eq!(fields.gender, gender);
        assert_eq!(fields.last_name, last_name);
    }

    #[derive(Deserialize, Debug)]
    struct Tuple(ManyFields);

    #[test]
    fn tuple_struct_err() {
        let url = Url::parse("http://127.0.0.1:8787/profile").unwrap();
        let params = CustomParam::new();
        let query = Query::<Tuple>::from_query_path(&url, &params, true);
        assert!(query.is_err());
    }

    #[derive(Deserialize, Debug)]
    struct Unit;

    #[test]
    // #[should_panic]
    fn unit_struct_err_panic() {
        let url = Url::parse("http://127.0.0.1:8787/profile?Hello=hello").unwrap();
        let params = CustomParam::new();
        let q = Query::<Unit>::from_query_path(&url, &params, true);
        assert!(q.is_err());
    }

    #[derive(Deserialize, Debug)]
    enum Enum {
        Hi,
    }

    #[test]
    fn enum_struct_err_panic() {
        let url = Url::parse("http://127.0.0.1:8787/profile?Hello=hello").unwrap();
        let params = CustomParam::new();
        let q = Query::<Enum>::from_query_path(&url, &params, true);
        assert!(q.is_err());
    }
}
