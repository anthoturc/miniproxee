use crate::error::{ErrorType, ProxyError, ProxyResult};
use crate::proxy::MiniProxee;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

pub struct Listener {
    inner: TcpListener,
    mini_proxee: Arc<MiniProxee>,
}

impl Listener {
    pub async fn new() -> ProxyResult<Listener> {
        let tcpl = TcpListener::bind("127.0.0.1:7777")
            .await
            .map_err(|_e| ProxyError::new(ErrorType::BindError("127.0.0.1:7777".to_string())))?;

        let mp = Arc::new(MiniProxee::default());
        Ok(Listener {
            inner: tcpl,
            mini_proxee: mp,
        })
    }
}

pub async fn run(l: Listener) -> ProxyResult<()> {
    loop {
        match l.inner.accept().await {
            Ok((conn, client_addr)) => {
                let mp = l.mini_proxee.clone();
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

async fn handle_conn(_conn: TcpStream, _mp: Arc<MiniProxee>) {
    todo!();
}
