use std::sync::Arc;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use chrono::{DateTime, NaiveDateTime, Utc};

#[derive(Debug, Deserialize, ToSchema)]

pub struct GameEntryCreate {
    pub game_name: String,
    pub description: String,
    pub game_rom: Vec<u8>,
}

#[derive(Debug, Deserialize, ToSchema)]

pub struct GameEntryPatch {
    pub game_name: Option<String>,
    pub description: Option<String>,
    pub game_rom: Option<Vec<u8>>,
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

pub async fn enforce_upload_permission(
    enforcer: Arc<RwLock<Enforcer>>,
    user: &str,
) -> Result<(), &'static str> {
    if enforcer.read().await.enforce((user, "game", "upload")).unwrap_or(false) {
        Ok(())
    } else {
        Err("Access denied")
    }
}

pub async fn insert_game_entry(pool: &PgPool, game_entry: &GameEntryData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO game_entries (game_id, game_name, author, description, created_at, updated_at, game_rom)
        VALUES ($1, $2, $3, $4, $5, $6, $7)",
        game_entry.game_id,
        game_entry.game_name,
        game_entry.author,
        game_entry.description,
        game_entry.created_at,
        game_entry.updated_at,
        game_entry.game_rom
    )
        .execute(pool)
        .await?;

    Ok(())
}

use axum::{debug_handler, Extension, Json};
use axum::extract::State;
use casbin::{CoreApi, Enforcer};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing_subscriber::fmt::format;
use utoipa::ToSchema;
use crate::{AppState, MaybeUserInfo};

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
#[debug_handler]
pub async fn create_game(
    State(app): State<Arc<AppState>>,
    Extension(user_info): Extension<MaybeUserInfo>,
    Json(game_input): Json<GameEntryCreate>,
) -> Result<(StatusCode, Json<GameEntryMetadata>), (StatusCode, String)> {
    // 1. Authorization: check if user can "create" a "game"
    let can_create = app.casbin
        .enforce_user_action(&user_info, "upload", "game")
        .await;

    if !can_create {
        app.casbin
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    // 2. We have a valid user (Some) at this point (otherwise can_create = false).
    let user = user_info.unwrap();

    // 3. Insert into the database
    //    `author` could store user.sub or user.preferred_username, whichever your schema expects
    let new_row = sqlx::query_as!(
        GameEntryData,
        r#"
        INSERT INTO game_entries (game_name, description, game_rom, author)
        VALUES ($1, $2, $3, $4)
        RETURNING
          game_id,
          game_name,
          author,
          description,
          created_at,
          updated_at,
          game_rom
        "#,
        game_input.game_name,
        game_input.description,
        game_input.game_rom,
        Uuid::parse_str(&user.sub).expect("Failed to parse user uuid"),
    )
        .fetch_one(&app.pg_pool)
        .await
        .map_err(|err| {
            eprintln!("DB Error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
        })?;

    // 4. Construct the response metadata (omitting the ROM)
    let metadata = GameEntryMetadata {
        game_id: new_row.game_id,
        game_name: new_row.game_name,
        author: new_row.author,
        description: new_row.description,
        created_at: new_row.created_at,
        updated_at: new_row.updated_at,
    };

    let game_darn = format!("game:{}", new_row.game_id);
    let new_role = format!("{}:author", game_darn);
    let _ = app.casbin.add_role_for_user(&user.sub, &new_role).await;
    let _ = app.casbin.add_allow_policy(&new_role, "*", &game_darn).await;

    // 5. Return 201 Created with the metadata
    Ok((StatusCode::CREATED, Json(metadata)))
}