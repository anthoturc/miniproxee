use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt};

// This simplified HTTP detection works by looking
// at the method at the start of the stream.
// Since URIs can be long, the HTTP version is not looked at or
// searched for in the stream. This balances performance with
// the desire to process HTTP middleware.
// The following methods are valid in HTTP/1.1
//
// GET
// POST
// PUT
// PATCH
// DELETE
// HEAD
// OPTIONS
// TRACE
// CONNECT
//
// The longest of these methods is 7 so we will
// perform a search against the first 7 bytes in the stream
// to determine if HTTP 1.1 is used.
const LARGEST_HTTP_METHOD: usize = 7;

const MAX_HTTP_DETECTION_SECONDS: u64 = 10;

// List of HTTP 1.1 methods
const H1_METHODS: [&str; 9] = [
    "GET", "PUT", "HEAD", "POST", "PATCH", "TRACE", "DELETE", "OPTIONS", "CONNECT",
];

pub async fn determine_http<Reader>(reader: &mut Reader) -> bool
where
    Reader: AsyncRead + Unpin,
{
    let mut maybe_method_bytes = BytesMut::with_capacity(LARGEST_HTTP_METHOD);
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(MAX_HTTP_DETECTION_SECONDS),
        reader.read_buf(&mut maybe_method_bytes),
    )
    .await
    {
        Ok(Ok(bytes_read)) => determine_http11(maybe_method_bytes, bytes_read),
        _ => false,
    }
}

fn determine_http11(buf: BytesMut, n: usize) -> bool {
    if n == 0 {
        return false;
    }
    for method in H1_METHODS {
        if method.as_bytes() == &buf[..method.len()] {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::detect::{determine_http, determine_http11};
    use bytes::BytesMut;
    use rstest::rstest;
    use tokio::io::AsyncReadExt;
    use tokio_test::io::{Builder, Mock};

    #[tokio::test]
    async fn test_determine_http() {
        let mut reader = Builder::new().read(b"GET /hello HTTP/1.1").build();

        assert!(determine_http(&mut reader).await);
        read_to_end(&mut reader).await;
    }

    #[rstest]
    #[case::get(b"GET /some/long/path?a=b HTTP/1.1", true)]
    #[case::post(b"POST /some/long/path?a=b HTTP/1.1", true)]
    #[case::put(b"PUT /some/long/path?a=b HTTP/1.1", true)]
    #[case::patch(b"PATCH /some/long/path?a=b HTTP/1.1", true)]
    #[case::head(b"HEAD /some/long/path?a=b HTTP/1.1", true)]
    #[case::options(b"OPTIONS /some/long/path?a=b HTTP/1.1", true)]
    #[case::connect(b"CONNECT /some/long/path?a=b HTTP/1.1", true)]
    #[case::trace(b"TRACE /some/long/path?a=b HTTP/1.1", true)]
    #[case::delete(b"DELETE /some/long/path?a=b HTTP/1.1", true)]
    #[case::got_prefix(b"GOTME /some/long/path?a=b HTTP/1.1", false)]
    #[case::post_prefix(b"POSTED /some/long/path?a=b HTTP/1.1", true)]
    #[case::invalid_prefix(b"LONGD /some/long/path?a=b HTTP/1.1", false)]
    fn test_determine_http11(#[case] input: &[u8], #[case] expected_http: bool) {
        assert_eq!(determine_http11(BytesMut::from(input), input.len()), expected_http);
    }

    // tokio test will panic if the reader has not been completely
    // read or written to
    async fn read_to_end(reader: &mut Mock) {
        let mut buf = BytesMut::with_capacity(100);
        loop {
            if let Ok(n) = reader.read_buf(&mut buf).await {
                if n == 0 {
                    break;
                }
            } else {
                break;
            }
        }
    }
}
