use std::{collections::HashMap, future::Future};

use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::http::*;

type Parameters = Vec<(String, String)>;

#[async_trait]
pub trait Handler<'a> {
    // @FIXME: We only handle the request headers now.
    async fn handle(&self, request: &'a RequestParts, params: Parameters) -> Option<Response>;
}

#[async_trait]
impl<'a, F, Fut, Res> Handler<'a> for F
where
    F: Fn(&RequestParts, Parameters) -> Fut + Sync,
    Fut: Future<Output = Res> + Send,
    Res: IntoResponse,
{
    async fn handle(&self, request: &RequestParts, params: Parameters) -> Option<Response> {
        Some(self(request, params).await.into_response())
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

// @TODO: Add tests
