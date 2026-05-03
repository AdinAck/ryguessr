use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use colors::Srgb8;

use crate::{Context, handle, room};

#[derive(serde::Deserialize, Default)]
pub struct InitRequest {
    username: Option<String>,
    color: Option<Srgb8>,
}

#[derive(serde::Serialize)]
pub struct InitResponse {
    room_id: room::Id,
    username: String,
    color: Srgb8,
}

pub async fn init_handler(
    State(context): State<Context>,
    jar: CookieJar,
    Json(request): Json<InitRequest>,
) -> Result<(CookieJar, Json<InitResponse>), StatusCode> {
    let (jar, client_id) = match jar.get(handle::ID_COOKIE_NAME) {
        Some(c) => {
            let client_id =
                handle::Id::try_from(c.value()).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (jar, client_id)
        }
        None => {
            let client_id = handle::Id::generate();
            (jar.add(client_id.to_cookie()), client_id)
        }
    };

    let location = context
        .engine
        .get_random_location()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut model = context.model.write().await;

    let username = request
        .username
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| model.generate_unique_name());

    let (room_id, color) =
        model.create_room(location, client_id.clone(), username.clone(), request.color);

    Ok((
        jar,
        Json(InitResponse {
            room_id,
            username,
            color,
        }),
    ))
}
