use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tracing::{debug, error, info};

mod trace;

#[tracing::instrument(skip(stream))]
async fn handle_connection(mut stream: TcpStream, addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    info!("Accepted connection from {addr}");

    let request = read_http_request(&mut stream).await?;

    debug!(
        "Incoming request: {}",
        request.split("\r\n").next().unwrap()
    );

    write_http_response(&mut stream).await?;

    info!("Closing connecetion");

    Ok(())
}

#[tracing::instrument(skip(stream))]
async fn write_http_response(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
    stream.write_all(b"\r\n").await?;
    stream.write_all(b"Thanks fastthanlime!\n").await?;

    Ok(())
}

#[tracing::instrument(skip(stream))]
async fn read_http_request(stream: &mut TcpStream) -> Result<String, Box<dyn Error>> {
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

    Ok(String::from_utf8(request)?)
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
