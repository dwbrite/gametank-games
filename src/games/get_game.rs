use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::{Path, State};
use http::StatusCode;
use uuid::Uuid;
use darn_authorize_macro::authorize;
use crate::AppState;
use crate::auth::KeycloakUserInfo;
use crate::darn::Darn;
use crate::games::{GameEntryData, GameEntryMetadata, GameEntryPatch, GAME_NS};
use crate::games::game_roles::GamePermissions;

#[utoipa::path(
    get,
    path = "/games/{game_id}",
    responses(
        (status = 200, description = "Game got successfully", body = GameEntryData),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    )
)]
#[authorize(GamePermissions::View, |game_id: &Uuid| { Darn::with_namespace(GAME_NS, &game_id.to_string()) })]
pub async fn get_game(
    State(app_state): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
    Path(game_id): Path<Uuid>,
) -> Result<(StatusCode, Json<GameEntryData>), (StatusCode, String)> {
    println!("hello?");
    let game_result = sqlx::query_as!(
        GameEntryData,
        r#"
        SELECT *
        FROM game_entries
        WHERE game_id = $1
        "#,
        game_id
    )
        .fetch_one(&app_state.pg_pool)
        .await;

    match game_result {
        Ok(game_entry) => {
            Ok((StatusCode::OK, Json(game_entry)))
        }
        Err(e) => {
            eprintln!("Error fetching games: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve games.".to_string(),
            ))
        }
    }
}