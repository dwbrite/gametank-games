use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use axum_core::__private::tracing::{debug, error};
use axum_core::__private::tracing::log::warn;
use itertools::Itertools;
use sqlx_adapter::SqlxAdapter;
use tokio::sync::Mutex;
use crate::MaybeUserInfo;
use casbin::{CoreApi, DefaultModel, Enforcer, Error as CasbinError, MgmtApi, RbacApi, Result as CasbinResult};
use casbin::error::RbacError;
use keycloak::types::Permission;
use strum::IntoEnumIterator;
use crate::auth::{RoleMarker, SiteRoles};
use crate::darn::{Darn, DarnRole, DarnSubject};

// TODO: update casbin functions to use multiple generics for Into<Darn> where there are multiple parameters <D1, D2>

pub struct Casbin {
    enforcer: Mutex<Enforcer>,
}

pub async fn init_casbin(database_url: String) -> Casbin {
    println!("init_casbin");
    let model = DefaultModel::from_str(include_str!("rbac_model.conf")).await.unwrap();
    let adapter = SqlxAdapter::new(database_url, 10).await.unwrap();
    let mut enforcer = Mutex::new(Enforcer::new(model, adapter).await.unwrap());

    let casbin = Casbin {
        enforcer,
    };

    println!("made it here!");
    SiteRoles::create_roles_in_namespace(&casbin, Darn::new("site")).await;

    casbin
}

impl Casbin {
    /// Example method to add a role to some subject (e.g., ("user:steve" -> "game:{id}:contributor") or "game:{id}:contributor").
    pub async fn add_subj_role(
        &self,
        subj: impl Into<DarnSubject>,
        role: impl Into<DarnRole>,
    ) -> CasbinResult<bool> {
        let subj = subj.into();
        let role = role.into();
        self.enforcer.lock().await.add_grouping_policy(vec![subj.to_string(), role.to_string()]).await
    }

    /// Example method to remove a role from the *current* user.
    pub async fn remove_subj_role(
        &self,
        subj: impl Into<DarnSubject>,
        role: impl Into<DarnRole>,
    ) -> CasbinResult<bool> {
        let subj = subj.into();
        let role = role.into();
        let mut guard = self.enforcer.lock().await;
        guard.delete_role_for_user(&subj.to_string(), &role.to_string(), None).await
    }

    /// Add a policy that allows `role` to do `action` on `object`.
    /// e.g. ("game:abc:author", "*", "game:abc").
    pub async fn add_allow_policy(
        &self,
        role: &DarnRole,
        action: &str, // TODO: maybe encode more about Actions later :/
        object: impl Into<Darn>,
    ) -> CasbinResult<bool> {
        let action = action.to_string();
        let object = object.into().to_string();
        let mut guard = self.enforcer.lock().await;
        guard.add_policy(vec![role.to_string(), action, object, "allow".to_string()]).await
    }

    /// Enforce that the *current* user can `action` a `resource`.
    /// If there's no user logged in, returns `false`.
    pub async fn enforce_action(
        &self,
        subj: impl Into<DarnSubject>,
        action: &str,
        resource: impl Into<Darn>,
    ) -> bool {
        let subj = subj.into();
        let resource = resource.into();
        let guard = self.enforcer.lock().await;
        let result = guard.enforce((&subj.to_string(), action, &resource.to_string()));
        if let Err(err) = &result {
            warn!("Failed to enforce permissions for {}. {}: {}", subj, action, err);
        }
        result.unwrap_or(false)
    }

    /// Retrieve all roles **directly** assigned to this user (no inheritance).
    /// Example: If user "alice" has "admin" role, returns ["admin"].
    pub async fn get_explicit_roles(&self, subj: impl Into<DarnSubject>,) -> Vec<String> {
        let mut guard = self.enforcer.lock().await;
        guard.get_roles_for_user(&subj.into().to_string(), None)
    }

    /// Retrieve all **implicit** roles for a user, including inherited roles.
    /// If "admin" inherits from "manager", and user "alice" has "admin",
    /// this returns ["admin", "manager"].
    pub async fn get_implicit_roles(&self, subj: impl Into<DarnSubject>,) -> Vec<String> {
        let mut guard = self.enforcer.lock().await;
        // `get_implicit_roles_for_user` can return an error; we just default to empty on error.
        guard.get_implicit_roles_for_user(&subj.into().to_string(), None)
    }

    /// Retrieve all **implicit** permissions for a user. That includes direct
    /// and inherited roles. Typically returns tuples of (subject, action, object).
    /// For example: [("site:admin", "\*", "\*"), ...]
    pub async fn get_implicit_permissions(
        &self,
        subj: impl Into<DarnSubject>,
    ) -> Vec<Vec<String>> {
        let mut guard = self.enforcer.lock().await;
        guard.get_implicit_permissions_for_user(&subj.into().to_string(), None)
    }
}
