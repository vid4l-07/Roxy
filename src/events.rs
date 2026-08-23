use crate::http;

pub enum TuiEvents {
    Forward(http::Request),
    SetIntercept(bool)
}

pub enum ProxyEvents {
    ReceivedRequest(http::Request)
}
