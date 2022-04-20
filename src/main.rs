use http::{BoxedBody, RequestParts};
use router::{Dispatcher, Parameters};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

use crate::http::ResponseParts;
use crate::router::{get, Router};

mod handler;
mod http;
mod router;
mod trace;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    trace::init()?;

    run_server().await
}

async fn run_server() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:4444").await?;

    info!("Listening at port :4444");

    let router = Arc::new(Router::new("/check", get(check)));

    loop {
        let (stream, addr) = listener.accept().await?;

        let router = Arc::clone(&router);

        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, addr, router).await {
                error!(%err, "Error handling connection");
            }
        });
    }
    Ok(())
}

#[tracing::instrument(skip(stream, router))]
async fn handle_connection<R: Dispatcher>(
    mut stream: TcpStream,
    addr: SocketAddr,
    router: Arc<R>,
) -> Result<(), Box<dyn Error>> {
    info!("Accepted connection from {addr}");

    let request = read_http_request(&mut stream).await?;

    //debug!("Incoming request: {:?}", request);

    let resp = router
        .dispatch(&request)
        .await
        .unwrap_or_else(|| http::Response {
            head: ResponseParts {
                status: http::Status::OK,
                version: http::Version::Http11,
                headers: HashMap::new(),
            },
            body: BoxedBody::empty(),
        });

    write_http_response(&mut stream, resp).await?;

    info!("Closing connecetion");

    Ok(())
}

#[tracing::instrument(skip(stream, response))]
async fn write_http_response(
    stream: &mut TcpStream,
    mut response: http::Response, // @TODO: Make this generic
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

//async fn check<'a>(request: &'a RequestParts, params: Parameters) -> &'static str {
async fn check<'a>() -> &'static str {
    "Thanks fasterthanlime!"
}
