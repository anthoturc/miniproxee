pub type ProxyResult<T> = Result<T, ProxyError>;

#[derive(Debug, Clone)]
pub struct ProxyError {
    etype: ErrorType,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    Bind(String),
    UpstreamConnect(String),
    Bidirectional(String),
    Internal(String),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.etype {
            ErrorType::Bind(e) => write!(f, "failed to bind to: {e}"),
            ErrorType::UpstreamConnect(e) => write!(f, "failed to connect upstream: {e}"),
            ErrorType::Bidirectional(e) => write!(
                f,
                "failed to copy data between upstream and downstream: {e}"
            ),
            ErrorType::Internal(e) => write!(f, "unexpected internal error: {e}"),
        }
    }
}

impl ProxyError {
    pub fn new(etype: ErrorType) -> ProxyError {
        ProxyError { etype }
    }
}
