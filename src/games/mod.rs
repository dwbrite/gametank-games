mod game_roles;

use std::sync::Arc;
use sqlx::FromRow;
use uuid::{Uuid};

use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, ToSchema)]

pub struct GameEntryCreate {
    pub game_name: String,
    pub description: String,
    pub game_rom: Vec<u8>,
}

#[derive(Debug, Deserialize, ToSchema)]

pub struct GameEntryPatch {
    pub _game_name: Option<String>,
    pub _description: Option<String>,
    pub _game_rom: Option<Vec<u8>>,
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

use axum::{Extension, Json};
use axum::extract::State;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::AppState;
use crate::auth::{DefaultNamespace, RoleMarker, SiteRoles};
use crate::auth::authn_keycloak::KeycloakUserInfo;
use crate::auth::SitePermissions::CreateGame;
use crate::darn::{Darn, DarnUser};
use crate::games::game_roles::GameRoles;

const GAME_NS: &str = "game";

#[utoipa::path(
    post,
    path = "/games",
    request_body = GameEntryCreate,
    responses(
        (status = 200, description = "Game uploaded successfully", body = GameEntryMetadata),
        (status = 403, description = "Access denied"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_game(
    State(app): State<Arc<AppState>>,
    Extension(user_info): Extension<KeycloakUserInfo>,
    Json(game_input): Json<GameEntryCreate>,
) -> Result<(StatusCode, Json<GameEntryMetadata>), (StatusCode, String)> {

    let user_darn: DarnUser = DarnUser::from(&user_info);

    let can_create = app.casbin
        .enforce_action(&user_darn, CreateGame, SiteRoles::default_namespace())
        .await;

    if !can_create {
        let roles = app.casbin.get_implicit_roles(&user_darn).await;
        let perms = app.casbin.get_implicit_permissions(&user_darn).await;
        eprintln!("User {} roles: {:?}", user_info.sub, roles);
        eprintln!("User {} perms: {:?}", user_info.sub, perms);

        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }
    
    let author_uuid = match Uuid::parse_str(&user_info.sub) {
        Ok(s) => { s }
        Err(_) => {
            return Err((StatusCode::FORBIDDEN, "Invalid user ID".to_string()));
        }
    };

    let new_id = Uuid::new_v4();
    let new_row = sqlx::query_as!(
        GameEntryData,
        r#"
        INSERT INTO game_entries (game_id, game_name, description, game_rom, author)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
          game_id,
          game_name,
          author,
          description,
          created_at,
          updated_at,
          game_rom
        "#,
        new_id,
        game_input.game_name,
        game_input.description,
        game_input.game_rom,
        author_uuid,
    )
        .fetch_one(&app.pg_pool)
        .await
        .map_err(|err| {
            eprintln!("DB Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
        })?;

    let metadata = GameEntryMetadata {
        game_id: new_row.game_id,
        game_name: new_row.game_name,
        author: new_row.author,
        description: new_row.description,
        created_at: new_row.created_at,
        updated_at: new_row.updated_at,
    };

    let game_darn = Darn::with_namespace(GAME_NS, &metadata.game_id.to_string());
    GameRoles::create_roles_in_namespace(&app.casbin, game_darn).await;

    // 5. Return 201 Created with the metadata
    Ok((StatusCode::CREATED, Json(metadata)))
}
