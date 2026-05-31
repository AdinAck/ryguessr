use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use colors::Srgb8;

use crate::{AppError, Context, handle, room};

#[derive(serde::Deserialize, Default)]
pub struct InitRequest {
    username: Option<String>,
    color: Option<Srgb8>,
}

#[derive(serde::Serialize)]
pub struct InitResponse {
    api_key: String,
    room_id: room::Id,
    username: String,
    color: Srgb8,
}

pub async fn init_handler(
    State(context): State<Context>,
    jar: CookieJar,
    Json(request): Json<InitRequest>,
) -> Result<(CookieJar, Json<InitResponse>), AppError> {
    let (jar, client_id) = match jar.get(handle::ID_COOKIE_NAME) {
        Some(c) => {
            let client_id = handle::Id::try_from(c.value()).map_err(|e| {
                AppError::Internal(anyhow::anyhow!("invalid client id cookie: {e}"))
            })?;
            (jar, client_id)
        }
        None => {
            let client_id = handle::Id::generate();
            (jar.add(client_id.to_cookie()), client_id)
        }
    };

    let location = context.engine.get_random_location().await?;

    let (room_id, username, color) = context
        .create_room(location, client_id.clone(), request.username, request.color)
        .await;

    Ok((
        jar,
        Json(InitResponse {
            api_key: context.api_key.to_string(),
            room_id,
            username,
            color: color.srgb,
        }),
    ))
}
