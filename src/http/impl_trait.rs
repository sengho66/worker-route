use worker::worker_sys::web_sys::ReadableStream;
use worker::worker_sys::web_sys::Response as WebSysResponse;
use worker::WebSocket;
use worker::{Response, ResponseBody};

pub trait ToResponse {
    fn into_response(self) -> Response;
}

impl ToResponse for Box<[u8]> {
    fn into_response(self) -> Response {
        Response::from_body(ResponseBody::Body(self.into()))
            .unwrap_or_else(|_| Response::empty().unwrap())
    }
}

impl ToResponse for ReadableStream {
    fn into_response(self) -> Response {
        WebSysResponse::new_with_opt_readable_stream(Some(&self))
            .map_or_else(|_| Response::empty().unwrap(), Into::into)
    }
}

impl ToResponse for Option<WebSocket> {
    fn into_response(self) -> Response {
        self.map_or_else(
            || Response::empty().unwrap(),
            |w| Response::empty().unwrap().with_websocket(Some(w)),
        )
    }
}

impl ToResponse for worker::Result<Response> {
    fn into_response(self) -> Response {
        self.unwrap_or_else(|_| Response::empty().unwrap())
    }
}
