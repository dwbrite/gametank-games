use std::env;
use std::sync::Arc;
use axum::extract::State;
use axum::middleware::Next;
use axum_core::extract::Request;
use axum_core::response::Response;
use http::header;
use keycloak::{KeycloakAdmin, KeycloakAdminToken};
use keycloak::types::TypeString;
use reqwest::Client;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use crate::AppState;
use crate::auth::{DefaultNamespace, SiteRoles};
use crate::darn::DarnUser;

pub struct KeycloakClient {
    pub admin: KeycloakAdmin,
    pub realm: String,
    pub client_uuid: String,
    client_url: String,
}

impl KeycloakClient {
    #[allow(clippy::expect_used, clippy::unwrap_used)] // expect/unwrap justified on initialization
    pub async fn init() -> Self {
        let url: String = env::var("KEYCLOAK_URL").unwrap();
        let user: String = env::var("KEYCLOAK_ADMIN_USER").unwrap();
        let password: String = env::var("KEYCLOAK_ADMIN_PASSWORD").unwrap();
        let client_url: String = env::var("KEYCLOAK_CLIENT_URL").unwrap();
        let realm = "gametank-games";
        let client_id = "authz-backend".into();

        let client = reqwest::Client::new();
        let token = KeycloakAdminToken::acquire(&url, &user, &password, &client).await.unwrap();
        let admin = KeycloakAdmin::new(&url, token.clone(), client);

        let backend_client = admin.realm_clients_get(realm, Some(client_id), None, None, None, None, None).await.unwrap().first().unwrap().clone();
        let client_uuid = backend_client.id.unwrap();

        KeycloakClient {
            admin,
            realm: realm.to_string(),
            client_uuid,
            client_url,
        }
    }

    pub async fn get_username(user_id: String) -> String {
        let keycloak = KeycloakClient::init().await;
        keycloak.admin.realm_users_with_user_id_get(
            &keycloak.realm,
            &user_id,
            None
        ).await.unwrap_or_default().username.unwrap_or(TypeString::from("unknown")).to_string()
    }
}

pub async fn authn_keycloak_middleware(
    State(app): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let client = Client::new();

    let maybe_token = request.headers().get(header::AUTHORIZATION).and_then(|h| h.to_str().ok());
    let mut user_info = KeycloakUserInfo {
        sub: "guest".to_string(),
        preferred_username: "guest".to_string(),
        email: "".to_string(),
    };

    let user = &DarnUser::from(&user_info);
    let roles = app.casbin.get_explicit_roles(user).await;
    if roles.is_empty() {
        app.casbin
            .add_subj_role(user, SiteRoles::Guest.to_darn_role())
            .await;
    }

    if let Some(token) = maybe_token {
        let token = token.trim_start_matches("Bearer ").to_string();
        let client_url = KeycloakClient::init().await.client_url;
        if let Ok(response) = client.get(&client_url).bearer_auth(token).send().await {
            if response.status().is_success() {
                #[allow(clippy::expect_used)]{
                    user_info = response.json::<KeycloakUserInfo>().await.expect("Successful response from keycloak implies we get UserInfo. If this isn't true, then, fuck me I guess.");
                }
                initialize_user_role(&app, &user_info).await;
            }
        }
    }

    request.extensions_mut().insert(user_info);

    next.run(request).await
}

// Rust function to check user login and assign 'user' role if first login
pub async fn initialize_user_role(
    app: &Arc<AppState>,
    user: &KeycloakUserInfo,
) {
    let admins = [
        DarnUser::from(&KeycloakUserInfo {
            sub: "6d93fb96-8dad-410e-880d-ed79ca568bc3".to_string(),
            preferred_username: "".to_string(), // ignored
            email: "".to_string(), // ignored
        })
    ];

    let user = &DarnUser::from(user);

    // Check if the user has a role
    let roles = app.casbin.get_explicit_roles(user).await;
    if roles.is_empty() {
        if admins.contains(user) {
            app.casbin
                .add_subj_role(user, SiteRoles::Admin.to_darn_role())
                .await;
        } else {
            app.casbin
                .add_subj_role(user, SiteRoles::User.to_darn_role())
                .await;
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct KeycloakUserInfo {
    pub sub: String,
    pub preferred_username: String,
    pub email: String,
}