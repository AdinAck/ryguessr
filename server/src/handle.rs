use axum::{extract::FromRequestParts, http::StatusCode};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use derive_more::{AsRef, Deref};
use uuid::Uuid;

use crate::{colors::PlayerColor, room};

/// A handle to a particular client.
pub struct Handle {
    /// The [`Room`](crate::Room) the associated client is participating in.
    pub room: room::Id,
    /// The chosen username of the associated client.
    pub username: String,
    /// The color assigned to the client, persisted across rooms.
    pub color: PlayerColor,
}

/// The unique identifier for a [`Handle`].
#[derive(Clone, AsRef, Debug, Deref, Hash, PartialEq, Eq)]
pub struct Id(pub String);

/// The header name used by the client to identify itself to the server.
pub const ID_COOKIE_NAME: &str = "client-id";

impl Id {
    pub fn generate() -> Self {
        Id(Uuid::new_v4().to_string())
    }

    pub fn to_cookie(&self) -> Cookie<'static> {
        Cookie::build((ID_COOKIE_NAME, self.0.clone()))
            .path("/")
            .http_only(true)
            // .secure(true)
            .build()
    }
}

impl TryFrom<&str> for Id {
    type Error = StatusCode;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Id(value.to_owned()))
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Id {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        jar.get(ID_COOKIE_NAME)
            .map(|c| Id(c.value().to_string()))
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
