use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

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
