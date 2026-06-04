use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum AppError {
    /// The request's client id is not registered (no init / expired room).
    UnknownClient,
    /// The addressed room does not exist.
    RoomNotFound,
    /// The action requires `State::Active` but the room is `State::Inactive`.
    RoomInactive,
    /// The requested username is already in use by another client.
    UsernameTaken,
    /// Anything unexpected — surfaced as 500, logged on the way out.
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let AppError::Internal(err) = &self {
            tracing::error!(error = %err, "internal server error");
        }
        let (status, message) = match self {
            AppError::UnknownClient => (StatusCode::UNAUTHORIZED, "unknown client"),
            AppError::RoomNotFound => (StatusCode::NOT_FOUND, "room not found"),
            AppError::RoomInactive => (StatusCode::CONFLICT, "room is inactive"),
            AppError::UsernameTaken => (StatusCode::CONFLICT, "username already taken"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
        };
        (status, message).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err)
    }
}
