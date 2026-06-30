use miniproxee::net::{self, Listener};
use miniproxee::proxy::{MiniProxee, MiniProxeeInner, Peer};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::sync::Semaphore;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .name("miniproxee")
        .build()
        .expect("failed to build tokio runtime");

    rt.block_on(async {
        let mp = MiniProxee {
            inner: Arc::new(MiniProxeeInner {
                next_peer: Arc::new(AtomicUsize::new(0)),
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
                conn_permits: Arc::new(Semaphore::new(100)),
            }),
            permit_fut: None,
            permit: None,
        };
        let l = Listener::new(mp).await.unwrap();
        match net::run(l).await {
            Ok(()) => {
                println!("shutting down miniproxee");
            }
            Err(e) => {
                panic!("failed to run miniproxee: {e:?}")
            }
        }
    })
}
