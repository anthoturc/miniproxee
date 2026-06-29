use crate::error::{ErrorType, ProxyError, ProxyResult};
use std::net;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll};
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tower_service::Service;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Peer {
    pub sock_addr: net::SocketAddr,
}

#[derive(Default)]
pub struct MiniProxee {
    pub upstream_peers: Vec<Peer>,

    pub next_peer: Mutex<usize>,
}

impl MiniProxee {
    fn select_peer(&mut self) -> ProxyResult<Peer> {
        let next_peer = self
            .next_peer
            .get_mut()
            .map_err(|e| ProxyError::new(ErrorType::Internal(e.to_string())))?;
        if *next_peer >= self.upstream_peers.len() {
            *next_peer = 0;
        }
        let peer = self.upstream_peers[*next_peer];
        *next_peer += 1;
        Ok(peer)
    }
}

impl Service<TcpStream> for MiniProxee {
    type Response = ();
    type Error = ProxyError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // todo: handle backpressure!
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, mut downstream_connection: TcpStream) -> Self::Future {
        let peer = match self.select_peer() {
            Ok(peer) => peer,
            Err(e) => {
                return Box::pin(async { Err(e) });
            }
        };
        Box::pin(async move {
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
    use crate::proxy::{MiniProxee, Peer};
    use std::sync::Mutex;

    #[test]
    fn test_select_next_peer() {
        let mut mp = MiniProxee {
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
            next_peer: Mutex::new(0usize),
        };

        let peer = mp.select_peer();
        assert!(peer.is_ok());
        let peer = peer.unwrap();
        assert_eq!(peer, mp.upstream_peers[0]);

        let peer = mp.select_peer();
        assert!(peer.is_ok());
        let peer = peer.unwrap();
        assert_eq!(peer, mp.upstream_peers[1]);

        let peer = mp.select_peer();
        assert!(peer.is_ok());
        let peer = peer.unwrap();
        assert_eq!(peer, mp.upstream_peers[2]);

        // the first peer should be selected when using Round Robin
        // peer selection
        let peer = mp.select_peer();
        assert!(peer.is_ok());
        let peer = peer.unwrap();
        assert_eq!(peer, mp.upstream_peers[0]);
    }
}
