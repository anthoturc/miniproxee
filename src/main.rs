use miniproxee::net::{self, Listener};
use miniproxee::proxy::{MiniProxee, Peer};
use std::sync;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .name("miniproxee")
        .build()
        .expect("failed to build tokio runtime");

    rt.block_on(async {
        let mp = MiniProxee {
            next_peer: sync::Mutex::new(0usize),
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
