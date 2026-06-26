use crate::net::Listener;

mod detect;
mod error;
mod net;
mod proxy;

fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .name("miniproxee")
        .build()
        .expect("failed to build tokio runtime");

    rt.block_on(async {
        let l = Listener::new().await.unwrap();
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
