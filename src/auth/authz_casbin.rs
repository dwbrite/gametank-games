use std::sync::Arc;
use axum_core::__private::tracing::error;
use itertools::Itertools;
use sqlx_adapter::SqlxAdapter;
use tokio::sync::Mutex;
use crate::{MaybeUserInfo, UserInfo};
use casbin::{CoreApi, DefaultModel, Enforcer, Error as CasbinError, MgmtApi, RbacApi, Result as CasbinResult};
use casbin::error::RbacError;

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
    /// Example method to add a role for the *current* user (e.g., "game:{id}:contributor").
    pub async fn add_role_for_user(
        &self,
        user: &UserInfo,
        role: &str,
    ) -> CasbinResult<bool> {
        let user_id = &user.sub;

        let mut guard = self.enforcer.lock().await;
        guard.add_role_for_user(user_id, role, None).await
    }

    /// Example method to remove a role from the *current* user.
    pub async fn remove_role_for_user(
        &self,
        user: &UserInfo,
        role: &str,
    ) -> CasbinResult<bool> {
        let user_id = &user.sub;

        let mut guard = self.enforcer.lock().await;
        guard.delete_role_for_user(user_id, role, None).await
    }

    /// Add a policy that allows `role` to do `action` on `object`.
    /// e.g. ("game:abc:author", "*", "game:abc").
    pub async fn add_allow_policy(
        &self,
        role: &str,
        action: &str,
        object: &str
    ) -> CasbinResult<bool> {
        let mut guard = self.enforcer.lock().await;
        guard.add_policy(vec![role, action, object, "allow"].into_iter().map_into().collect::<Vec<String>>()).await
    }

    /// Enforce that the *current* user can `action` a `resource`.
    /// If there's no user logged in, returns `false`.
    pub async fn enforce_user_action(
        &self,
        user: &UserInfo,
        action: &str,
        resource: &str
    ) -> bool {
        let user_id = &user.sub;
        let guard = self.enforcer.lock().await;
        let result = guard.enforce((user_id, action, resource));
        println!("Enforce user action: {:?}", result);
        result.unwrap_or(false)
    }

    /// Retrieve all roles **directly** assigned to this user (no inheritance).
    /// Example: If user "alice" has "admin" role, returns ["admin"].
    pub async fn get_roles_for_user(&self, user: &UserInfo) -> Vec<String> {
        let user_id = &user.sub; // "guest" if not logged in, or actual user ID if logged in
        let mut guard = self.enforcer.lock().await;
        guard.get_roles_for_user(user_id, None)
    }

    /// Retrieve all **implicit** roles for a user, including inherited roles.
    /// If "admin" inherits from "manager", and user "alice" has "admin",
    /// this returns ["admin", "manager"].
    pub async fn get_implicit_roles_for_user(&self, user: &UserInfo) -> Vec<String> {
        let user_id = &user.sub;
        let mut guard = self.enforcer.lock().await;
        // `get_implicit_roles_for_user` can return an error; we just default to empty on error.
        guard.get_implicit_roles_for_user(user_id, None)
    }

    /// Retrieve all **implicit** permissions for a user. That includes direct
    /// and inherited roles. Typically returns tuples of (subject, action, object).
    /// For example: [("admin", "*", "games"), ...]
    pub async fn get_implicit_permissions_for_user(
        &self,
        user: &UserInfo
    ) -> Vec<Vec<String>> {
        let user_id = &user.sub;
        let mut guard = self.enforcer.lock().await;
        guard.get_implicit_permissions_for_user(user_id, None)
        // Depending on your policy_definition, you might have
        // (sub, act, obj) or (sub, act, obj, eft). Adjust your tuple as needed.
    }
}
