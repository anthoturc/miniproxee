use crate::error::{ErrorType, ProxyError};
use std::net;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tokio::sync::{AcquireError, OwnedSemaphorePermit, Semaphore};
use tower::Service;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Peer {
    pub sock_addr: net::SocketAddr,
}

#[derive(Clone)]
pub struct MiniProxeeInner {
    // Effectively static list of upstream peers
    pub upstream_peers: Vec<Peer>,

    // Counter of the round robin "next peer"
    // for a given downstream connection
    pub next_peer: Arc<AtomicUsize>,

    // Semaphore to track allowing connections
    // into the proxy.
    pub conn_permits: Arc<Semaphore>,
}

pub struct MiniProxee {
    pub inner: Arc<MiniProxeeInner>,
    pub permit_fut:
        Option<Pin<Box<dyn Future<Output = Result<OwnedSemaphorePermit, AcquireError>> + Send>>>,
    pub permit: Option<OwnedSemaphorePermit>,
}

impl MiniProxee {
    fn select_peer(&self) -> Peer {
        let next_peer =
            self.inner.next_peer.fetch_add(1, Ordering::Relaxed) % self.inner.upstream_peers.len();

        self.inner.upstream_peers[next_peer]
    }
}

impl Service<TcpStream> for MiniProxee {
    type Response = ();
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    // This implementation of poll ready leans on the future that
    // the semaphore acquire_owned returns. The future returned by the semaphore
    // will handle the wasker in the supplied context.
    // We can assume that when a permit becomes available, the task will be woken
    // correctly.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut fut = self
            .permit_fut
            .take()
            .unwrap_or_else(|| Box::pin(self.inner.conn_permits.clone().acquire_owned()));

        let poll_res = match fut.as_mut().poll(cx) {
            Poll::Ready(r) => match r {
                Ok(p) => {
                    self.permit.replace(p);
                    Poll::Ready(Ok(()))
                }
                Err(_) => {
                    return Poll::Ready(Err(ProxyError::new(ErrorType::Internal(
                        "sempahore was closed!".into(),
                    ))));
                }
            },
            Poll::Pending => {
                self.permit_fut.replace(fut);
                Poll::Pending
            }
        };

        poll_res
    }

    fn call(&mut self, mut downstream_connection: TcpStream) -> Self::Future {
        let peer = self.select_peer();
        // this assumes poll_ready has been invoked and
        // the driver of the service is behaving
        let permit = self
            .permit
            .take()
            .expect("failed to retrieve permit, was poll_ready called?");
        Box::pin(async move {
            // we want drop behavior when the future completes
            let _permit = permit;
            let mut upstream_connection = match TcpStream::connect(peer.sock_addr).await {
                Ok(upstream_connection) => upstream_connection,
                Err(e) => {
                    return Err(ProxyError::new(ErrorType::UpstreamConnect(e.to_string())));
                }
            };

            // todo: honestly handing this off isn't great in my opinion since I don't
            //  have as much control over the connection lifecycle or metric tracking...
            // todo: copy with sized could be configured on a per-upstream or client basis
            // todo: metrics for rx and tx bytes should come from here too
            copy_bidirectional(&mut downstream_connection, &mut upstream_connection)
                .await
                .map_err(|e| ProxyError::new(ErrorType::Bidirectional(e.to_string())))
                .map(|_| ())
        })
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::Semaphore;

    use crate::proxy::{MiniProxee, MiniProxeeInner, Peer};
    use std::sync::{Arc, atomic::AtomicUsize};

    #[test]
    fn test_select_next_peer() {
        let mp = MiniProxee {
            inner: Arc::new(MiniProxeeInner {
                upstream_peers: vec![
                    Peer {
                        sock_addr: "127.0.0.1:8081".parse().unwrap(),
                    },
                    Peer {
                        sock_addr: "127.0.0.1:8082".parse().unwrap(),
                    },
                    Peer {
                        sock_addr: "127.0.0.1:8083".parse().unwrap(),
                    },
                ],
                next_peer: Arc::new(AtomicUsize::new(0)),
                conn_permits: Arc::new(Semaphore::new(100)),
            }),
            permit_fut: None,
            permit: None,
        };

        let peer = mp.select_peer();
        assert_eq!(peer, mp.inner.upstream_peers[0]);

        let peer = mp.select_peer();
        assert_eq!(peer, mp.inner.upstream_peers[1]);

        let peer = mp.select_peer();
        assert_eq!(peer, mp.inner.upstream_peers[2]);

        // the first peer should be selected when using Round Robin
        // peer selection
        let peer = mp.select_peer();
        assert_eq!(peer, mp.inner.upstream_peers[0]);
    }
}
