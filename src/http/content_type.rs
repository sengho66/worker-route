use std::fmt::Display;

use http::HeaderValue;
use mime::Mime;

use paste::paste;

/// `Content-Type` header, defined in [RFC 9110 §8.3](https://datatracker.ietf.org/doc/html/rfc9110#section-8.3).
#[derive(Debug)]
pub struct ContentType(Mime, &'static str, Option<&'static str>);

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // safety: to_header_value method is evaluated at compiled time
        // it will not panic 100%
        write!(f, "{}", self.to_header_value().to_str().unwrap())
    }
}

macro_rules! impl_content_types {
    ($(($name:ident, $content:ident, $lit:literal, $val:expr);)+) => {
        impl ContentType {
            $(
                paste! {
                    #[doc = " `Content-Type: " $lit "` header."]
                    pub const fn $name() -> Self {
                        Self(mime::$content, $lit, $val)
                    }
                }
            )+

            #[inline]
            /// Returns a &str of the Mime's
            pub fn as_str(&self) -> &str {
                self.1
            }

            #[inline]
            /// Returns `self`' Mime
            pub fn get_mime(&self) -> &Mime {
                &self.0
            }

            #[inline]
            /// Returns `self`'s charset if there's any.
            pub fn char_set(&self) -> Option<&'static str> {
                self.2
            }

            /// Returns a `HeaderValue` from `self`'s top level media type with its subtype
            /// and charset if there's any.
            ///
            /// # Example
            /// ```
            /// let html_utf8 = ContentType::html_utf8();
            /// let header_value = html_utf8.to_header_value();
            ///
            /// assert_eq!(header_value, "text/html; charset=utf-8");
            /// ```
            pub const fn to_header_value(&self) -> http::HeaderValue {
                HeaderValue::from_static(self.1)
            }
        }
        $(
            paste! {
                #[test]
                fn [<$name _test>]() {
                    // ensures panics won't happen, there's an unwrap in the display trait
                    assert!(!ContentType::$name().to_string().is_empty());
                    // constructing a HeaderValue from static has a possibility of panic
                    // only if the HeaderValue is invalid or contains empty spaces
                    // to_header_value guarantees it'll never panic
                    assert_eq!(ContentType::$name().to_header_value(), $lit);
                    // or if users decide to construct a HeaderValue manually
                    // let v = HeaderValue::from_str(ContentType::$name().as_str();
                    assert!(HeaderValue::from_str(ContentType::$name().as_str()).is_ok());
                    assert_eq!(
                        HeaderValue::from_str(ContentType::$name().as_str()).unwrap(),
                        ContentType::$name().to_header_value()
                    );
                }
            }
        )+
    }
}

impl_content_types! {
    (form_url_encoded, APPLICATION_WWW_FORM_URLENCODED, "application/x-www-form-urlencoded", None);
    (html_utf8, TEXT_HTML_UTF_8, "text/html; charset=utf-8", Some("charset=utf-8"));
    (html, TEXT_HTML, "text/html", None);
    (jpeg, IMAGE_JPEG, "image/jpeg", None);
    (json, APPLICATION_JSON, "application/json", None);
    (octet_stream, APPLICATION_OCTET_STREAM, "application/octet-stream", None);
    (plaintext_utf8, TEXT_PLAIN_UTF_8, "text/plain; charset=utf-8", Some("charset=utf-8"));
    (plaintext, TEXT_PLAIN, "text/plain", None);
    (png, IMAGE_PNG, "image/png", None);
    (xml, TEXT_XML, "text/xml", None);
}
