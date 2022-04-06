use http::{RequestParts, Response, ResponseParts};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::io::{copy, AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tracing::{debug, error, info};

mod http;
mod trace;

trait IntoResponse<B: AsyncRead> {
    fn into_response(self) -> Response<B>;
}

impl IntoResponse<&'static [u8]> for &'static str {
    fn into_response(self) -> Response<&'static [u8]> {
        //Response::builder().body(self).unwrap()
        Response {
            head: ResponseParts {
                status: http::Status::OK,
                version: http::Version::Http11,
                headers: HashMap::new(),
            },
            body: http::Body {
                inner: self.as_bytes(),
            },
        }
    }
}

#[tracing::instrument(skip(stream))]
async fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    info!("Accepted connection from {addr}");

    let request = read_http_request(&mut stream).await?;

    debug!("Incoming request: {:?}", request);

    write_http_response(&mut stream, "Thanks fasterthanlime!\n".into_response()).await?;

    info!("Closing connecetion");

    Ok(())
}

#[tracing::instrument(skip(stream, response))]
async fn write_http_response<B: AsyncRead + Unpin>(
    stream: &mut TcpStream,
    mut response: http::Response<B>, // @TODO: Make this generic
) -> Result<(), Box<dyn Error>> {
    // @TODO: This should be called status line or something
    let request_line = format!(
        "{:?} {} {}\r\n",
        response.version(),
        response.status().as_str(),
        response.status().reason(),
    );

    stream.write_all(request_line.as_bytes()).await?; // Assume UTF-8
                                                      // @TODO: Write request headers
    stream.write_all(b"\r\n").await?; // End headers

    // Why B must be Unpin?
    copy(&mut response.body.inner, stream).await?;

    Ok(())
}

#[tracing::instrument(skip(stream))]
async fn read_http_request(stream: &mut TcpStream) -> Result<RequestParts, Box<dyn Error>> {
    let mut request = vec![];

    loop {
        let mut buf = vec![0u8; 1024];
        let read = stream.read(&mut buf).await?;

        // EOF, or the client sent 0 bytes.
        if read == 0 {
            break;
        }

        request.extend_from_slice(&buf[..read]);

        if request.len() > 4 && &request[request.len() - 4..] == b"\r\n\r\n" {
            break;
        }
    }

    let request = std::str::from_utf8(&request)?;
    Ok(RequestParts::from_str(request)?)
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:4444").await?;

    info!("Listening at port :4444");

    loop {
        let (stream, addr) = listener.accept().await?;

        //drop(stream);
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, addr).await {
                error!(%err, "Error handling connection");
            }
        });
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    trace::init()?;

    run_server().await
}
