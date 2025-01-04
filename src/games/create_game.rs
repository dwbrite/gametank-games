use axum::extract::State;
use std::sync::Arc;
use axum::{Extension, Json};
use http::StatusCode;
use sqlx::query_as;
use uuid::Uuid;
use darn_authorize_macro::authorize;
use crate::AppState;
use crate::auth::{DefaultNamespace, KeycloakUserInfo, RoleMarker, SiteRoles};
use crate::auth::SitePermissions::CreateGame;
use crate::darn::{Darn};
use crate::games::{GameEntryCreate, GameEntryMetadata, GAME_NS};
use crate::games::game_roles::GameRoles;

// #[utoipa::path(
//     post,
//     path = "/games",
//     request_body = GameEntryCreate,
//     responses(
//         (status = 201, description = "Game uploaded successfully", body = GameEntryMetadata),
//         (status = 403, description = "Access denied"),
//         (status = 500, description = "Internal server error")
//     )
// )]

#[authorize(SitePermissions::CreateGame, || { SiteRoles::default_namespace() })]
pub async fn create_game(
    State(app_state): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
    Json(input): Json<GameEntryCreate>,
) -> Result<(StatusCode, Json<GameEntryMetadata>), (StatusCode, String)> {
    let author_uuid = Uuid::parse_str(&user_info.sub).map_err(|_| {
        (StatusCode::FORBIDDEN, "Invalid user ID".to_string())
    })?;

    let new_id = Uuid::new_v4();
    let metadata = query_as!(
        GameEntryMetadata,
        r#"
        INSERT INTO game_entries (game_id, game_name, description, game_rom, author)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
          game_id,
          game_name,
          description,
          author,
          created_at,
          updated_at
        "#,
        new_id,
        input.game_name,
        input.description,
        input.game_rom,
        author_uuid,
    )
        .fetch_one(&app_state.pg_pool)
        .await
        .map_err(|err| {
            eprintln!("DB Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
        })?;

    let game_darn = Darn::with_namespace(GAME_NS, &metadata.game_id.to_string());
    GameRoles::create_roles_in_namespace(&app_state.casbin, game_darn).await;

    Ok((StatusCode::CREATED, Json(metadata)))
}
//
// pub struct GameEntryCreateEndpoint;
// #[async_trait::async_trait]
// impl AuthorizedHandler<Json<GameEntryCreate>, (StatusCode, Json<GameEntryMetadata>)> for GameEntryCreateEndpoint {
//     async fn authorize(
//         app_state: &Arc<AppState>,
//         user_info: &KeycloakUserInfo,
//         _input: &Json<GameEntryCreate>,
//     ) -> Result<(), (StatusCode, String)> {
//         let site_darn = SiteRoles::default_namespace();
//
//         app_state
//             .casbin
//             .enforce_http(user_info, CreateGame, site_darn)
//             .await
//     }
//
//     async fn process(
//         app_state: Arc<AppState>,
//         user_info: KeycloakUserInfo,
//         input: Json<GameEntryCreate>,
//     ) -> Result<(StatusCode, Json<GameEntryMetadata>), (StatusCode, String)> {
//         let author_uuid = Uuid::parse_str(&user_info.sub).map_err(|_| {
//             (StatusCode::FORBIDDEN, "Invalid user ID".to_string())
//         })?;
//
//         let new_id = Uuid::new_v4();
//         let metadata = sqlx::query_as!(
//             GameEntryMetadata,
//             r#"
//             INSERT INTO game_entries (game_id, game_name, description, game_rom, author)
//             VALUES ($1, $2, $3, $4, $5)
//             RETURNING
//               game_id,
//               game_name,
//               description,
//               author,
//               created_at,
//               updated_at
//             "#,
//             new_id,
//             input.game_name,
//             input.description,
//             input.game_rom,
//             author_uuid,
//         )
//             .fetch_one(&app_state.pg_pool)
//             .await
//             .map_err(|err| {
//                 eprintln!("DB Error: {:?}", err);
//                 (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
//             })?;
//
//         let game_darn = Darn::with_namespace(GAME_NS, &metadata.game_id.to_string());
//         GameRoles::create_roles_in_namespace(&app_state.casbin, game_darn).await;
//
//         Ok((StatusCode::CREATED, Json(metadata)))
//     }
// }