use axum::http::HeaderName;
use axum_extra::headers::Header;
use derive_more::{AsRef, Deref};

use crate::room;

/// A handle to a particular client.
pub struct Handle {
    /// The [`Room`](crate::Room) the associated client is participating in.
    pub room: room::Id,
    /// The chosen username of the associated client.
    username: String,
}

/// The unique identifier for a [`Handle`].
#[derive(AsRef, Deref, Hash, PartialEq, Eq)]
pub struct Id(String);

/// The header name used by the client to identify itself to the server.
static ID_HEADER_NAME: HeaderName = HeaderName::from_static("client-id");

impl Header for Id {
    fn name() -> &'static HeaderName {
        &ID_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;
        Ok(Id(value
            .to_str()
            .map_err(|_| axum_extra::headers::Error::invalid())?
            .to_owned()))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, _: &mut E) {
        // not needed, the server will never send this header to the client
        unimplemented!()
    }
}
