use std::env;
use std::sync::Arc;
use axum::middleware::Next;
use axum_core::extract::Request;
use axum_core::response::Response;
use http::header;
use keycloak::{KeycloakAdmin, KeycloakAdminToken};
use keycloak::types::ResourceRepresentation;
use reqwest::Client;
use crate::UserInfo;

pub struct KeycloakClient {
    pub admin: KeycloakAdmin,
    pub realm: String,
    pub client_uuid: String,
}

pub type Keycloak = Arc<KeycloakClient>;

pub async fn init_keycloak() -> Keycloak {
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

    Arc::new(
        KeycloakClient {
            admin,
            realm: realm.to_string(),
            client_uuid,
        }
    )
}

pub async fn authn_keycloak_middleware(mut request: Request, next: Next) -> Response {
    let client = Client::new();
    // TODO: update url
    let url = "https://keycloak.dwbrite.com/realms/gametank-games/protocol/openid-connect/userinfo";

    let maybe_token = request.headers().get(header::AUTHORIZATION).and_then(|h| h.to_str().ok());
    let mut user_info = None;


    if let Some(token) = maybe_token {
        let token = token.trim_start_matches("Bearer ").to_string();
        if let Ok(response) = client.get(url).bearer_auth(token).send().await {
            println!("response {:?}", response);
            if response.status().is_success() {
                println!("user info: {:?}", response.headers());
                user_info = response.json::<UserInfo>().await.ok();
            }
        }
    }

    request.extensions_mut().insert(user_info);

    let response = next.run(request).await;

    response
}