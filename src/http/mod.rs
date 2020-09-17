//! A helper module that supports HTTP

mod batch;
mod graphiql_source;
#[cfg(feature = "multipart")]
mod multipart;
mod playground_source;
mod websocket;

pub use batch::receive_batch;
pub use graphiql_source::graphiql_source;
#[cfg(feature = "multipart")]
pub use multipart::MultipartOptions;
pub use playground_source::{playground_source, GraphQLPlaygroundConfig};
pub use websocket::WebSocketStream;

use crate::{ParseRequestError, Request};
use futures::io::AsyncRead;
use futures::AsyncReadExt;
use multipart::receive_multipart;

/// Receive a GraphQL request from a content type and body.
pub async fn receive(
    content_type: Option<impl AsRef<str>>,
    body: impl AsyncRead + Send + 'static,
    opts: MultipartOptions,
) -> Result<Request, ParseRequestError> {
    #[cfg(feature = "multipart")]
    if let Some(Ok(boundary)) = content_type.map(multer::parse_boundary) {
        return receive_multipart(body, boundary, opts).await;
    }
    receive_json(body).await
}

/// Receive a GraphQL request from a body as JSON.
async fn receive_json(body: impl AsyncRead + Send + 'static) -> Result<Request, ParseRequestError> {
    let mut data = Vec::new();
    futures::pin_mut!(body);
    body.read_to_end(&mut data)
        .await
        .map_err(ParseRequestError::Io)?;
    Ok(serde_json::from_slice::<Request>(&data).map_err(ParseRequestError::InvalidRequest)?)
}
