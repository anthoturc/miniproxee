use std::time::Duration;

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio::time::Instant;

mod util;

#[tokio::test]
async fn test_transparent_proxy() {
    util::init();

    let mut client_conn = TcpStream::connect("127.0.0.1:7777")
        .await
        .expect("failed to connect to test-miniproxee");

    let to_send = b"yellooooo";
    client_conn
        .write_all(to_send)
        .await
        .expect("write to proxy should succeed");
    let mut buf = BytesMut::with_capacity(1024);
    let n = client_conn.read_buf(&mut buf).await.unwrap();
    assert_eq!(n, to_send.len());
    assert!(client_conn.shutdown().await.is_ok());
}

#[tokio::test(start_paused = true)]
async fn test_timeout() {
    util::init();

    let mut client_conn = TcpStream::connect("127.0.0.1:7777")
        .await
        .expect("failed to connect to test-miniproxee");

    let to_send = b"yolooooo";
    client_conn
        .write_all(to_send)
        .await
        .expect("write to proxy should succeed");
    let mut buf = BytesMut::with_capacity(1024);
    // inject "fake" sleep to hit timeout limit
    // it's "fake" because the start_paused macro
    // arg will pause time and forward it by 11 seconds
    // for the test.
    tokio::time::sleep(Duration::from_secs(11)).await;
    let read_res = client_conn.read_buf(&mut buf).await;
    assert!(read_res.is_ok());
    // inject "fake" sleep to hit timeout limit
    // it's "fake" because the start_paused macro
    // arg will pause time and forward it by 11 seconds
    // for the test.
    tokio::time::sleep(Duration::from_secs(11)).await;
    let mut buf = BytesMut::with_capacity(1024);
    match client_conn.read_buf(&mut buf).await {
        Ok(n) => assert_eq!(n, 0), // the buffer has a good capacity but nothing is going to be read here
        Err(_) => {}
    }
}

#[ignore = "this is more for perf checking than correctness"]
#[tokio::test]
async fn test_mutli_client_proxy() {
    util::init();

    let mut js = JoinSet::new();
    let start = Instant::now();
    let bytes_per_client = 1000 as u128;
    let clients = 100;
    for _ in 0..clients {
        let client_to_send = "a".repeat(bytes_per_client as usize);
        let fut = async move {
            let mut client_conn = TcpStream::connect("127.0.0.1:7777").await.unwrap();
            let mut count = 20;
            loop {
                if count <= 0 {
                    break;
                }
                client_conn
                    .write_all(client_to_send.as_bytes())
                    .await
                    .expect("write to proxy should succeed");
                let mut buf = BytesMut::with_capacity(1024);
                let n = client_conn.read_buf(&mut buf).await.unwrap();
                assert_eq!(n, client_to_send.len());
                count -= 1;
            }
            assert!(client_conn.shutdown().await.is_ok());
        };
        js.spawn(fut);
    }

    js.join_all().await;
    let end = Instant::now();

    let ms = end.duration_since(start).as_millis();
    let bytes_sent = 20 * clients * bytes_per_client;
    println!("duration: {} ms", ms);
    println!("bytes sent: {} bytes", bytes_sent);
    println!("throughput: {} bytes/s", bytes_sent / 1000 * ms)
}
