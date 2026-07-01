use crate::error::{ErrorType, ProxyError, ProxyResult};
use crate::middleware::timeout::Timeout;
use crate::proxy::MiniProxee;
use std::fmt::Debug;
use std::time::Duration;
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
                let timeout = Timeout::new(Duration::from_secs(10), mp);
                tokio::spawn(async move {
                    handle_conn(conn, timeout).await;
                });
            }
            Err(e) => {
                // todo: log metric and error?
                println!("failed to accept connection: {e:?}");
            }
        }
    }
}

async fn handle_conn<T>(mut conn: TcpStream, mut s: T)
where
    T: Service<TcpStream>,
    T::Error: Debug,
{
    let call_fut = {
        match s.ready().await {
            Ok(proxee) => proxee.call(conn),
            Err(_) => {
                let _ = conn.shutdown().await;
                return;
            }
        }
    };
    match call_fut.await {
        Ok(_) => {}
        Err(e) => {
            println!("error proxying connection: {e:?}");
        }
    }
}
