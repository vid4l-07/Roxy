use crate::http;

pub enum TuiEvents {
    Forward{
        id: usize,
        request: http::Request
    },
    SetIntercept(bool),
    SendRepeater { 
        index: usize ,
        request: http::Request
    },
}

pub enum ProxyEvents {
    ReceivedRequest {
        id: usize,
        request: http::Request,
    },
    Error(String),
    FatalError(String),
    RepeaterResponse {
        index: usize,
        response: http::Response
    }

}
