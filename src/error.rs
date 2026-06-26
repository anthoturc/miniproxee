pub type ProxyResult<T> = Result<T, ProxyError>;

#[derive(Debug, Clone)]
pub struct ProxyError {
    etype: ErrorType,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    BindError(String),
    AcceptError(String),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.etype {
            ErrorType::BindError(e) => write!(f, "failed to bind to: {e}"),
            ErrorType::AcceptError(e) => write!(f, "failed to accept: {e}"),
        }
    }
}

impl ProxyError {
    pub fn new(etype: ErrorType) -> ProxyError {
        ProxyError { etype }
    }
}
