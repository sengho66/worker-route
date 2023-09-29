use super::response::ResponseBuilder;
use crate::Error;

use http::{HeaderName, HeaderValue};
use std::iter::Map;
use std::ops::{Deref, DerefMut};
use worker::js_sys::{Array, IntoIter};
use worker::wasm_bindgen::JsValue;
use worker::Headers;

type HeaderPair = (HeaderName, HeaderValue);
pub enum HeadersOp {
    Insert,
    Append,
}

// pub struct HeaderName(pub(crate) HeaderName_);
// pub struct HeaderValue(pub(crate) HeaderValue_);

impl HeadersOp {
    pub fn set(&self, header: &HeaderPair, builder: &mut ResponseBuilder) {
        let res = match self {
            HeadersOp::Insert => builder.headers.set(&header.0, &header.1),
            HeadersOp::Append => builder.headers.append(&header.0, &header.1),
        };

        if let Err(err) = res {
            builder.error(err);
        }
    }
}

/// An wrapper for [`worker::Headers`](https://docs.rs/worker/latest/worker/struct.Headers.html) with additional methods.
///
/// This comes with two additional method which are [`self.len()`](Self::len) and [`self.is_empty()`](Self::is_empty).
#[derive(Clone, Debug, Default)]
pub struct HttpHeaders(pub(crate) Headers);

impl HttpHeaders {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns all the values of a header within a `Headers` object with a given name.
    ///
    /// # Panics
    ///
    /// Panics if [`HeaderName`] is constructed from using the method [`from_static`](HeaderName::from_static)
    /// and the static string is an invalid header or contains spaces.
    ///
    /// Eg: Header contains invalid header's name or spaces.
    ///
    pub fn get(&self, name: &HeaderName) -> Option<String> {
        self.0.get(name.as_str()).unwrap_or(None)
    }

    /// Returns a boolean stating whether a `Headers` object contains a certain header.
    ///
    /// # Panics
    ///
    /// Panics if [`HeaderName`] is constructed from using the method [`from_static`](HeaderName::from_static)
    /// and the static string is an invalid header or contains spaces.
    ///
    /// Eg: Header contains invalid header's name or spaces.
    pub fn has(&self, name: &HeaderName) -> bool {
        self.0.has(name.as_str()).unwrap_or(false)
    }

    /// Returns the number of elements in the headers.
    pub fn len(&self) -> usize {
        self.0.keys().count()
    }

    /// Returns `true` if the headers contain no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Append a header, keeping any that were set with an equivalent field name.
    ///
    /// # Errors
    ///
    /// Errors are returned if the header name or value is invalid (e.g. contains spaces)
    ///
    /// # Panics
    ///
    /// Panics if [`HeaderName`] or [`HeaderValue`] is constructed from using the method [`from_static`](HeaderName::from_static)
    /// and the static string is an invalid header or contains spaces.
    ///
    /// Eg: Header contains invalid header's name or spaces.
    pub fn append(&mut self, name: &HeaderName, value: &HeaderValue) -> Result<(), Error> {
        self.0.append(name.as_str(), value.to_str()?)?;
        Ok(())
    }

    /// Sets a new value for an existing header inside a `Headers` object, or adds the header if it does not already exist.
    ///
    /// # Errors
    ///
    /// Errors are returned if the header name or value is invalid (e.g. contains spaces)
    ///
    /// # Panics
    ///
    /// Panics if [`HeaderName`] or [`HeaderValue`] is constructed from using the method [`from_static`](HeaderName::from_static)
    /// and the static string is an invalid header or contains spaces.
    ///
    /// Eg: Header contains invalid header's name or spaces.
    pub fn set(&mut self, name: &HeaderName, value: &HeaderValue) -> Result<(), Error> {
        self.0.set(name.as_str(), value.to_str()?)?;
        Ok(())
    }

    /// Deletes a header from a `Headers` object.
    ///
    /// # Errors
    /// Errors are returned if the header name or value is invalid (e.g. contains spaces)
    /// or if the JS Headers object's guard is immutable (e.g. for an incoming request)
    ///
    /// # Panics
    ///
    /// Panics if [`HeaderName`] is constructed from using the method [`from_static`](HeaderName::from_static)
    /// and the static string is an invalid header or contains spaces.
    ///
    /// Eg: Header contains invalid header's name or spaces.
    pub fn delete(&mut self, name: &HeaderName) -> Result<(), Error> {
        self.0.delete(name.as_str())?;
        Ok(())
    }

    /// Returns an iterator allowing to go through all key/value pairs contained in this object.
    pub fn entries(&self) -> HeaderIterator {
        self.0.entries()
    }

    /// Returns an iterator allowing you to go through all keys of the key/value pairs contained in
    /// this object.
    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.0.keys()
    }

    /// Returns an iterator allowing you to go through all values of the key/value pairs contained
    /// in this object.
    pub fn values(&self) -> impl Iterator<Item = String> {
        self.0.values()
    }
}

type F1 = fn(Result<JsValue, JsValue>) -> Array;
type HeaderIterator = Map<Map<IntoIter, F1>, fn(Array) -> (String, String)>;

impl IntoIterator for &HttpHeaders {
    type Item = (String, String);
    type IntoIter = HeaderIterator;

    fn into_iter(self) -> Self::IntoIter {
        self.0.entries()
    }
}

impl<'a> From<&'a mut HttpHeaders> for &'a mut Headers {
    fn from(headers: &'a mut HttpHeaders) -> Self {
        &mut headers.0
    }
}

macro_rules! impl_headers {
    ($from:ty, $to:ty) => {
        impl From<$from> for $to {
            fn from(headers: $from) -> Self {
                Self(headers)
            }
        }
    };

    ($from:ty, $to:ty, $headers:tt, $expr:tt) => {
        impl From<$from> for $to {
            fn from(headers: $from) -> Self {
                $headers(headers.$expr())
            }
        }
    };

    ($from:ty, $to:ty, $headers:ident, $expr:expr) => {
        impl From<$from> for $to {
            fn from($headers: $from) -> Self {
                $expr
            }
        }
    };
}

impl_headers!(Headers, HttpHeaders);
impl_headers!(&Headers, HttpHeaders, Self, clone);
impl_headers!(&HttpHeaders, Headers, headers, headers.0.clone());
impl_headers!(HttpHeaders, Headers, headers, headers.0);

impl Deref for HttpHeaders {
    type Target = Headers;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HttpHeaders {
    fn deref_mut(&mut self) -> &mut Headers {
        &mut self.0
    }
}
