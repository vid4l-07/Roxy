use crate::http;

pub enum TuiEvents {
    Forward(http::Request),
    SetIntercept(bool),
    SendRepeater { 
        index: usize ,
        request: http::Request
    },
}

pub enum ProxyEvents {
    ReceivedRequest(http::Request),
    Error(String),
    FatalError(String),
    RepeaterResponse {
        index: usize,
        response: http::Response
    }

}
