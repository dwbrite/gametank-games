use std::env;
use std::sync::Arc;
use axum::debug_handler;
use axum::extract::State;
use axum::middleware::Next;
use axum_core::__private::tracing::warn;
use axum_core::extract::Request;
use axum_core::response::Response;
use http::header;
use keycloak::{KeycloakAdmin, KeycloakAdminToken};
use keycloak::types::ResourceRepresentation;
use reqwest::Client;
use uuid::Uuid;
use crate::{AppState, UserInfo};

pub struct KeycloakClient {
    pub admin: KeycloakAdmin,
    pub realm: String,
    pub client_uuid: String,
}

pub async fn init_keycloak() -> KeycloakClient {
    let url: String = env::var("KEYCLOAK_URL").unwrap().into();
    let user: String = env::var("KEYCLOAK_ADMIN_USER").unwrap().into();
    let password: String = env::var("KEYCLOAK_ADMIN_PASSWORD").unwrap().into();
    let realm = "gametank-games";
    let client_id = "authz-backend".into();

    let client = reqwest::Client::new();
    let admin_token = KeycloakAdminToken::acquire(&url, &user, &password, &client).await.unwrap();
    let admin = KeycloakAdmin::new(&url, admin_token, client);

    let backend_client = admin.realm_clients_get(realm, Some(client_id), None, None, None, None, None).await.unwrap().first().unwrap().clone();
    let client_uuid = backend_client.id.unwrap();

    KeycloakClient {
        admin,
        realm: realm.to_string(),
        client_uuid,
    }
}


pub async fn authn_keycloak_middleware(
    State(app): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let client = Client::new();
    // TODO: update url
    let url = "https://keycloak.dwbrite.com/realms/gametank-games/protocol/openid-connect/userinfo";

    let maybe_token = request.headers().get(header::AUTHORIZATION).and_then(|h| h.to_str().ok());
    let mut user_info = None;


    if let Some(token) = maybe_token {
        let token = token.trim_start_matches("Bearer ").to_string();
        if let Ok(response) = client.get(url).bearer_auth(token).send().await {
            if response.status().is_success() {
                user_info = response.json::<UserInfo>().await.ok();
                match &user_info {
                    None => { warn!("no user info :(("); }
                    Some(user_info) => {
                        let uid = user_info.sub.parse().unwrap();
                        // checking if user is in db
                        check_first_login(&app, uid).await;
                    }
                }
            }
        }
    }

    request.extensions_mut().insert(user_info);

    let response = next.run(request).await;

    response
}

// Rust function to check user login and assign 'user' role if first login
pub async fn check_first_login(app: &Arc<AppState>, user_id: Uuid) -> anyhow::Result<()> {
    warn!("check first login");
    let exists = sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM user_login WHERE user_id = $1)",
        user_id
    )
        .fetch_one(&app.pg_pool)
        .await?;

    println!("exists: {:?}", exists);
    if !exists.unwrap_or(false) {
        println!("no user info found; inserting entry");
        // First login: insert into user_login and assign 'user' role
        sqlx::query!(
            "INSERT INTO user_login (user_id, last_login) VALUES ($1, NOW())",
            user_id
        )
            .execute(&app.pg_pool)
            .await?;

        // Add user to 'user' role in Casbin
        println!("inserted entry; adding user to user group");
        app.casbin.add_role_for_user(&user_id.to_string(), "user").await?;
    }

    Ok(())
}