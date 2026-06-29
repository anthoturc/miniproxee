use crate::error::{ErrorType, ProxyError, ProxyResult};
use crate::proxy::MiniProxee;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tower_service::Service;

pub struct Listener {
    inner: TcpListener,
    mini_proxee: Arc<Mutex<MiniProxee>>,
}

impl Listener {
    pub async fn new(mini_proxee: MiniProxee) -> ProxyResult<Listener> {
        let tcp_listener = TcpListener::bind("127.0.0.1:7777")
            .await
            .map_err(|_e| ProxyError::new(ErrorType::Bind("127.0.0.1:7777".to_string())))?;

        let mp = Arc::new(Mutex::new(mini_proxee));
        Ok(Listener {
            inner: tcp_listener,
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

async fn handle_conn(conn: TcpStream, mp: Arc<Mutex<MiniProxee>>) {
    let call_fut = {
        let mut guard = mp.lock().await;
        guard.call(conn)
    };
    match call_fut.await {
        Ok(()) => {}
        Err(e) => {
            println!("error proxying connection: {e:?}");
        }
    }
}
