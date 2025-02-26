mod game_roles;
pub mod create_game;
pub mod patch_game;
pub mod get_game;

use std::sync::Arc;
use axum::{Extension, Json};
use axum::extract::State;
use sqlx::FromRow;
use utoipa::ToSchema;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use http::StatusCode;
use keycloak::types::TypeString;
use darn_authorize_macro::authorize;
use crate::AppState;
use crate::auth::{DefaultNamespace, KeycloakUserInfo, SitePermissions, SiteRoles};

#[derive(Debug, Deserialize, ToSchema)]

pub struct GameEntryCreate {
    pub game_name: String,
    pub description: String,
    pub game_rom: Vec<u8>,
    pub public_access: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct GameEntryPatch {
    pub _game_name: Option<String>,
    pub _description: Option<String>,
    pub _game_rom: Option<Vec<u8>>,
    pub _public_access: Option<bool>,
}

#[derive(Debug, FromRow, ToSchema, Serialize, Deserialize)]
pub struct GameEntryData {
    pub game_id: Uuid,
    pub game_name: String,
    pub author: Uuid,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub game_rom: Vec<u8>,
    pub public_access: bool,
}


#[derive(Debug, FromRow, ToSchema, Serialize, Deserialize)]
pub struct GameEntryMetadata {
    pub game_id: Uuid,
    pub game_name: String,
    pub author: Uuid,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GameEntryMetadata {
    pub async fn humanize(self, app_state: &Arc<AppState>) -> GameEntryMetadataDisplay {
        let result = app_state.keycloak.admin.realm_users_with_user_id_get(
            &app_state.keycloak.realm,
            &self.author.to_string(),
            None
        ).await.unwrap_or_default().username.unwrap_or(TypeString::from("unknown")).to_string();

        GameEntryMetadataDisplay {
            metadata: self,
            author_name: result,
        }
    }
}

#[derive(Debug, FromRow, ToSchema, Serialize, Deserialize)]
pub struct GameEntryMetadataDisplay {
    metadata: GameEntryMetadata,
    author_name: String,
}

const GAME_NS: &str = "game";

#[utoipa::path(
    get,
    path = "/games",
    responses(
        (status = 200, description = "Here's a list of games!!!", body = [GameEntryMetadata]),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
#[authorize(SitePermissions::ViewPublic, || { SiteRoles::default_namespace() })]
pub async fn list_public_games(
    State(app_state): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
) -> Result<(StatusCode, Json<Vec<GameEntryMetadataDisplay>>), (StatusCode, String)> {
    let games_result = sqlx::query_as!(
        GameEntryMetadata,
        r#"
        SELECT
            game_id,
            game_name,
            description,
            author,
            created_at,
            updated_at
        FROM
            game_entries
        WHERE public_access
        ORDER BY
            created_at DESC
        "#
    )
        .fetch_all(&app_state.pg_pool)
        .await;


    use futures::future::join_all;

    match games_result {
        Ok(games) => {
            let games: Vec<GameEntryMetadata> = games;
            let g2 = join_all(
                games.into_iter().map(|entry| entry.humanize(&app_state))
            ).await;

            Ok((StatusCode::OK, Json(g2)))
        },
        Err(e) => {
            // Log the error for debugging purposes
            eprintln!("Error fetching games: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve games.".to_string(),
            ))
        }
    }
}

