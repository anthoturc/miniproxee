use std::{fmt::Display, time::Duration};

use std::pin::Pin;
use std::task::Poll;
use tokio::time::{Instant, timeout_at};
use tower::Service;

use crate::error::{ErrorType, ProxyError};

pub struct Timeout<S> {
    inner: S,
    timeout: Duration,
    timeout_at: Option<Instant>,
}

impl<S> Timeout<S> {
    pub fn new(timeout: Duration, inner: S) -> Self {
        Self {
            inner: inner,
            timeout: timeout,
            timeout_at: None,
        }
    }
}

impl<S, T> Service<T> for Timeout<S>
where
    S: Service<T>,
    S::Future: Send + 'static,
    S::Error: Display,
{
    type Response = S::Response;
    type Error = ProxyError;
    type Future =
        Pin<Box<dyn Future<Output = Result<<S as Service<T>>::Response, ProxyError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.timeout_at.is_none() {
            self.timeout_at.replace(Instant::now() + self.timeout);
        }
        let now = Instant::now();
        if now > self.timeout_at.as_ref().unwrap().clone() {
            Poll::Ready(Err(ProxyError::new(ErrorType::Timeout)))
        } else {
            self.inner.poll_ready(cx).map_err(|_| {
                ProxyError::new(ErrorType::Internal("failed to poll inner service".into()))
            })
        }
    }

    fn call(&mut self, req: T) -> Self::Future {
        let inner_call = self.inner.call(req);
        let timeout = self.timeout_at.unwrap();
        let t_fut = async move {
            match timeout_at(timeout, inner_call).await {
                Ok(r) => r.map_err(|e| ProxyError::new(ErrorType::Internal(e.to_string()))),
                Err(_) => Err(ProxyError::new(ErrorType::Timeout)),
            }
        };
        Box::pin(t_fut)
    }
}
