use miniproxee::error::ProxyResult;
use miniproxee::net;
use miniproxee::net::Listener;
use miniproxee::proxy::{MiniProxee, MiniProxeeInner, Peer};
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::channel;
use std::sync::{Arc, LazyLock};
use tokio::runtime::Builder;
use tokio::sync::Semaphore;

static PROXY: LazyLock<()> = LazyLock::new(|| {
    run_proxy().expect("proxy should not error");
});

pub fn run_proxy() -> ProxyResult<()> {
    // based on the configuration
    // in docker-compose.yml
    let (tx, rx) = channel::<()>();
    std::thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_all()
            .name("test-miniproxee")
            .build()
            .unwrap();
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

        let res = rt.block_on(async {
            let lt = match Listener::new(mp).await {
                Ok(l) => l,
                Err(e) => {
                    return Err(e);
                }
            };
            // signal that connections can take place.
            match tx.send(()) {
                Ok(()) => {}
                Err(e) => {
                    println!("failed to send message: {e:?}");
                }
            }
            net::run(lt).await
        });
        if res.is_err() {
            println!("{:?}", res.err().unwrap());
        }
    });

    if let Err(e) = rx.recv() {
        println!("got error while receiving message on proxy startup: {e:?}");
    }

    Ok(())
}

pub fn init() {
    let _ = &*PROXY;
}
