use crate::error::{ErrorType, ProxyError, ProxyResult};
use crate::proxy::MiniProxee;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tower::{Service, ServiceExt};

pub struct Listener {
    inner: TcpListener,
    mini_proxee: MiniProxee,
}

impl Listener {
    pub async fn new(mini_proxee: MiniProxee) -> ProxyResult<Listener> {
        let tcp_listener = TcpListener::bind("127.0.0.1:7777")
            .await
            .map_err(|_e| ProxyError::new(ErrorType::Bind("127.0.0.1:7777".to_string())))?;

        let mp = mini_proxee;
        Ok(Listener {
            inner: tcp_listener,
            mini_proxee: mp,
        })
    }
}

pub async fn run(l: Listener) -> ProxyResult<()> {
    loop {
        match l.inner.accept().await {
            Ok((conn, _client_addr)) => {
                let mp = MiniProxee {
                    inner: l.mini_proxee.inner.clone(),
                    permit_fut: None,
                    permit: None,
                };
                tokio::spawn(async move {
                    handle_conn(conn, mp).await;
                });
            }
            Err(e) => {
                // todo: log metric and error?
                println!("failed to accept connection: {e:?}");
            }
        }
    }
}

async fn handle_conn(mut conn: TcpStream, mut mp: MiniProxee) {
    let call_fut = {
        match mp.ready().await {
            Ok(proxee) => proxee.call(conn),
            Err(_) => {
                let _ = conn.shutdown().await;
                return;
            }
        }
    };
    match call_fut.await {
        Ok(()) => {}
        Err(e) => {
            println!("error proxying connection: {e:?}");
        }
    }
}
