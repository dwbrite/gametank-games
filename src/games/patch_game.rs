use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::{Path, State};
use http::StatusCode;
use uuid::Uuid;
use darn_authorize_macro::authorize;
use crate::AppState;
use crate::auth::KeycloakUserInfo;
use crate::darn::Darn;
use crate::games::{GameEntryMetadata, GameEntryPatch, GAME_NS};
use crate::games::game_roles::GamePermissions;

#[utoipa::path(
    post,
    path = "/games/{game_id}",
    request_body = GameEntryPatch,
    responses(
        (status = 200, description = "Game updated successfully", body = GameEntryMetadata),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    )
)]
#[authorize(GamePermissions::Patch, |game_id: &Uuid| { Darn::with_namespace(GAME_NS, &game_id.to_string()) })]
pub async fn patch_game(
    State(app_state): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
    Path(game_id): Path<Uuid>,
    Json(input): Json<GameEntryPatch>,
) -> Result<(StatusCode, Json<GameEntryMetadata>), (StatusCode, String)> {
    
    

    Err((StatusCode::OK, "Ugh".to_string()))
}