use super::{
    header::{HeaderName, HeaderValue, COOKIE, SET_COOKIE},
    headers::HeadersOp,
    response::ResponseBuilder,
    HttpHeaders,
};

#[derive(Clone, Copy)]
pub enum CookieHelper {
    Set,
    Get,
}

impl CookieHelper {
    pub fn get(&self, headers: &HttpHeaders) -> impl Iterator<Item = cookie::Cookie<'_>> {
        headers
            .entries()
            .filter(|(k, _)| k == self.header_name().as_str())
            .flat_map(|(_, v)| cookie::Cookie::parse(v))
    }

    pub fn set(&self, builder: &mut ResponseBuilder, cookie: &cookie::Cookie<'_>) {
        HeadersOp::Append.set(
            &(
                self.header_name(),
                HeaderValue::from_str(&cookie.to_string()).unwrap(),
            ),
            builder,
        );
    }

    fn header_name(&self) -> HeaderName {
        match self {
            CookieHelper::Set => SET_COOKIE,
            CookieHelper::Get => COOKIE,
        }
    }
}
