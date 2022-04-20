use std::{collections::HashMap, future::Future, io::Cursor};

use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::{http::*, router::Parameters};

#[async_trait]
pub trait Handler {
    // @FIXME: We only handle the request headers now.
    async fn handle(&self, params: Parameters) -> Option<Response>;
}

#[async_trait]
impl<F, Fut, Res> Handler for F
where
    F: Fn(Parameters) -> Fut + Send + Sync, // @FIXME: 'static?
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
{
    async fn handle(&self, params: Parameters) -> Option<Response> {
        Some(self(params).await.into_response())
    }
}

pub trait IntoResponse {
    type BodyType: AsyncRead + Sync + Send;
    fn into_response(self) -> Response;
}

//impl<'a> IntoResponse for &'a str {
//type BodyType = &'a [u8];
impl IntoResponse for &'static str {
    type BodyType = &'static [u8];
    fn into_response(self) -> Response {
        Response {
            head: ResponseParts {
                status: Status::OK,
                version: Version::Http11,
                headers: HashMap::new(),
            },
            body: BoxedBody {
                inner: Box::new(self.as_bytes()),
            },
        }
    }
}

impl IntoResponse for String {
    type BodyType = &'static [u8];
    fn into_response(self) -> Response {
        Response {
            head: ResponseParts {
                status: Status::OK,
                version: Version::Http11,
                headers: HashMap::new(),
            },
            body: BoxedBody {
                inner: Box::new(Cursor::new(self)),
            },
        }
    }
}

// @TODO: Add tests
