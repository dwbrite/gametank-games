use std::sync::Arc;
use sqlx::{FromRow, PgPool};

type UserId = String; // actually a uuid
type GameId = String;

#[derive(FromRow)]
pub struct GameEntry {
    pub id: GameId,
    pub name: String,
    pub description: String,
    pub author: UserId,
    pub rom: String, // TODO: placeholders
    pub thumbnail: String,
    pub created_at: String,
    pub updated_at: String,
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

pub async fn insert_game_entry(pool: &PgPool, game: &GameEntry) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO game_entries (id, name, description, author, rom, thumbnail, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
        .bind(&game.id)
        .bind(&game.name)
        .bind(&game.description)
        .bind(&game.author)
        .bind(&game.rom)
        .bind(&game.thumbnail)
        .bind(&game.created_at)
        .bind(&game.updated_at)
        .execute(pool)
        .await?;
    Ok(())
}

use axum::{Extension, Json};
use casbin::{CoreApi, Enforcer};
use tokio::sync::RwLock;
use uuid::Uuid;
//
// #[utoipa::path(
//     post,
//     path = "/games",
//     request_body = GameEntry,
//     responses(
//         (status = 200, description = "Game uploaded successfully", body = GameEntry),
//         (status = 403, description = "Access denied"),
//         (status = 500, description = "Internal server error")
//     )
// )]
// pub async fn upload_game(
//     Extension(pool): Extension<PgPool>,
//     Extension(enforcer): Extension<Arc<RwLock<Enforcer>>>,
//     user: String, // Extracted from authentication middleware
//     Json(mut game): Json<GameEntry>,
// ) -> Result<Json<GameEntry>, &'static str> {
//     // Check authorization
//     enforce_upload_permission(enforcer, &user).await?;
//
//     // Set game metadata
//     let now = Utc::now().to_rfc3339();
//     game.id = Uuid::new_v4().to_string();
//     game.author = user;
//     game.created_at = now.clone();
//     game.updated_at = now;
//
//     // Insert into database
//     insert_game_entry(&pool, &game).await.map_err(|_| "Database error")?;
//
//     Ok(Json(game))
// }

