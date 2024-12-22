use std::fmt::Display;
use std::sync::Arc;
use axum_core::__private::tracing::{debug, error};
use axum_core::__private::tracing::log::warn;
use itertools::Itertools;
use sqlx_adapter::SqlxAdapter;
use tokio::sync::Mutex;
use crate::{MaybeUserInfo};
use casbin::{CoreApi, DefaultModel, Enforcer, Error as CasbinError, MgmtApi, RbacApi, Result as CasbinResult};
use casbin::error::RbacError;
use crate::auth::site_roles::{add_site_roles, SITE_NS};
use crate::darn::Darn;

pub struct Casbin {
    enforcer: Mutex<Enforcer>,
}

pub trait HasPermissions<P> where P: Display {
    fn permissions(&self) -> Vec<P>;
}

pub trait IntoDarnWithContext: Display {
    fn to_darn(&self, parent: &Darn) -> Darn {
        parent.new_child(&self.to_string())
    }
}

pub async fn apply_role_policies<'a, T, P>(
    casbin: &'a Casbin,
    obj_darn: &'a Darn,
    roles: &'static [T]
) where
    T: IntoDarnWithContext + HasPermissions<P>,
    P: Display + 'a + 'static
{
    for role in roles {
        for action in role.permissions() {
            let role_darn = &role.to_darn(obj_darn);
            let action = &action.to_string();
            match casbin.add_allow_policy(role_darn, action, obj_darn).await {
                Ok(_) => {}
                Err(_) => { warn!("Failed to apply casbin policy on role {}. (This role has likely been initialized already? Maybe I should delete the roles first??? sus.)", role); }
            }
        }
    }
}


pub async fn init_casbin(database_url: String) -> Casbin {
    println!("init_casbin");
    let model = DefaultModel::from_str(include_str!("rbac_model.conf")).await.unwrap();
    let adapter = SqlxAdapter::new(database_url, 10).await.unwrap();
    let mut enforcer = Mutex::new(Enforcer::new(model, adapter).await.unwrap());

    let casbin = Casbin {
        enforcer,
    };

    add_site_roles(&casbin, &SITE_NS.into()).await;

    casbin
}

impl Casbin {
    /// Example method to add a role for the *current* user (e.g., "game:{id}:contributor").
    pub async fn add_subj_role(
        &self,
        subj: &Darn,
        role: &Darn,
    ) -> CasbinResult<bool> {
        self.enforcer.lock().await.add_grouping_policy(vec![subj.to_string(), role.to_string()]).await
    }

    /// Example method to remove a role from the *current* user.
    pub async fn remove_subj_role(
        &self,
        subj: &Darn,
        role: &Darn,
    ) -> CasbinResult<bool> {
        let mut guard = self.enforcer.lock().await;
        guard.delete_role_for_user(&subj.to_string(), &role.to_string(), None).await
    }

    /// Add a policy that allows `role` to do `action` on `object`.
    /// e.g. ("game:abc:author", "*", "game:abc").
    pub async fn add_allow_policy(
        &self,
        role: &Darn,
        action: &str, // TODO: maybe encode more about Actions later :/
        object: &Darn,
    ) -> CasbinResult<bool> {
        let role = role.to_string();
        let action = action.to_string();
        let object = object.to_string();
        let mut guard = self.enforcer.lock().await;
        guard.add_policy(vec![role, action, object, "allow".to_string()]).await
    }

    /// Enforce that the *current* user can `action` a `resource`.
    /// If there's no user logged in, returns `false`.
    pub async fn enforce_action(
        &self,
        subj: &Darn,
        action: &str,
        resource: &Darn
    ) -> bool {
        let guard = self.enforcer.lock().await;
        let result = guard.enforce((&subj.to_string(), action, &resource.to_string()));
        if let Err(err) = &result {
            warn!("Failed to enforce permissions for {}. {}: {}", subj, action, err);
        }
        result.unwrap_or(false)
    }

    /// Retrieve all roles **directly** assigned to this user (no inheritance).
    /// Example: If user "alice" has "admin" role, returns ["admin"].
    pub async fn get_explicit_roles(&self, subj: &Darn) -> Vec<String> {
        let mut guard = self.enforcer.lock().await;
        guard.get_roles_for_user(&subj.to_string(), None)
    }

    /// Retrieve all **implicit** roles for a user, including inherited roles.
    /// If "admin" inherits from "manager", and user "alice" has "admin",
    /// this returns ["admin", "manager"].
    pub async fn get_implicit_roles(&self, subj: &Darn) -> Vec<String> {
        let mut guard = self.enforcer.lock().await;
        // `get_implicit_roles_for_user` can return an error; we just default to empty on error.
        guard.get_implicit_roles_for_user(&subj.to_string(), None)
    }

    /// Retrieve all **implicit** permissions for a user. That includes direct
    /// and inherited roles. Typically returns tuples of (subject, action, object).
    /// For example: [("site:admin", "\*", "\*"), ...]
    pub async fn get_implicit_permissions(
        &self,
        subj: &Darn
    ) -> Vec<Vec<String>> {
        let mut guard = self.enforcer.lock().await;
        guard.get_implicit_permissions_for_user(&subj.to_string(), None)
    }
}
