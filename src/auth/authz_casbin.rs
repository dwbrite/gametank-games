use std::sync::Arc;
use axum_core::__private::tracing::error;
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi, RbacApi};
use itertools::Itertools;
use sqlx_adapter::SqlxAdapter;
use tokio::sync::Mutex;
use crate::MaybeUserInfo;

pub struct Casbin {
    enforcer: Mutex<Enforcer>,
}

    pub async fn init_casbin(database_url: String) -> Casbin {
    let model = DefaultModel::from_str(include_str!("rbac_model.conf")).await.unwrap();
    let adapter = SqlxAdapter::new(database_url, 10).await.unwrap();
    let mut enforcer = Mutex::new(Enforcer::new(model, adapter).await.unwrap());

    let casbin = Casbin {
        enforcer,
    };

    // TODO: default roles
    // we explicitly ignore these errors
    let _ = casbin.add_allow_policy("user", "upload", "game").await;
    casbin
}

impl Casbin {
    // TODO: return types,

    /// Example method to add a role for a user (e.g., "game:{id}:contributor").
    pub async fn add_role_for_user(&self, user_id: &str, role: &str) -> Result<bool, casbin::Error> {
        let mut guard = self.enforcer.lock().await;
        guard.add_role_for_user(user_id, role, None).await
    }

    /// Example method to remove a role from a user.
    pub async fn remove_role_for_user(&self, user_id: &str, role: &str) -> Result<bool, casbin::Error> {
        let mut guard = self.enforcer.lock().await;
        guard.delete_role_for_user(user_id, role, None).await
    }

    pub async fn add_allow_policy(&self, role: &str, action: &str, object: &str) -> casbin::Result<bool> {
        let mut guard = self.enforcer.lock().await;
        guard.add_policy(vec![role, action, object, "allow"].into_iter().map_into().collect::<Vec<String>>()).await
    }

    pub async fn enforce_user_action(
        &self,
        user: &MaybeUserInfo,
        action: &str,
        resource: &str
    ) -> bool {
        // If there's no logged-in user, automatically deny
        let Some(user_info) = user else {
            return false;
        };

        // Lock the enforcer for thread-safe use
        let guard = self.enforcer.lock().await;

        // `sub` is the user's unique ID from Keycloak
        let user_id = &user_info.sub;
        guard.enforce((user_id, action, resource)).unwrap_or_else(|_| false)
    }
}