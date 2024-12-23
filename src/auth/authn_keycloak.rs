use std::env;
use std::sync::Arc;
use axum::debug_handler;
use axum::extract::State;
use axum::middleware::Next;
use axum_core::__private::tracing::{info, warn};
use axum_core::extract::Request;
use axum_core::response::Response;
use http::header;
use keycloak::{KeycloakAdmin, KeycloakAdminToken};
use keycloak::types::ResourceRepresentation;
use reqwest::Client;
use uuid::Uuid;
use crate::{AppState, MaybeUserInfo, UserInfo};
use crate::auth::{RoleMarker, DefaultNamespace, SiteRoles};
// use crate::auth::SiteRoles::{Admin, Guest, User};
use crate::darn::{Darn, DarnUser};

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
    let mut user_info = UserInfo {
        sub: "guest".to_string(),
        preferred_username: "guest".to_string(),
        email: "".to_string(),
    };


    if let Some(token) = maybe_token {
        let token = token.trim_start_matches("Bearer ").to_string();
        if let Ok(response) = client.get(url).bearer_auth(token).send().await {
            if response.status().is_success() {
                user_info = response.json::<UserInfo>().await.unwrap(); // TODO: we unwrap?!?!?!?
                check_user_roles(&app, &user_info).await;
            }
        }
    }

    request.extensions_mut().insert(user_info);

    let response = next.run(request).await;

    response
}

// Rust function to check user login and assign 'user' role if first login
pub async fn check_user_roles(
    app: &Arc<AppState>,
    user: &UserInfo,
) -> anyhow::Result<()> {
    let admins = [
        DarnUser::from(&UserInfo {
            sub: "6d93fb96-8dad-410e-880d-ed79ca568bc3".to_string(),
            preferred_username: "".to_string(), // ignored
            email: "".to_string(), // ignored
        })
    ];

    let user = &DarnUser::from(user);

    // Check if the user has a role
    let roles = app.casbin.get_explicit_roles(user).await;
    info!("Current roles: {:?}", roles);

    if roles.is_empty() {
        if admins.contains(user) {
            info!("User is an admin; assigning 'site:admin' role");
            app.casbin
                .add_subj_role(user, SiteRoles::Admin.to_darn_role())
                .await?;
        } else {
            info!("User is not an admin; assigning 'site:user' role");
            app.casbin
                .add_subj_role(user, SiteRoles::User.to_darn_role())
                .await?;
        }
    } else {
        println!("User already has roles; no changes made");
    }

    Ok(())
}
