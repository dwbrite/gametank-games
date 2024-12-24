use axum_core::__private::tracing::log::warn;
use sqlx_adapter::SqlxAdapter;
use tokio::sync::Mutex;
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi, RbacApi};
use log::error;
use crate::auth::{DefaultNamespace, PermissionMarker, SiteRoles};
use crate::darn::{Darn, DarnRole, DarnSubject};

pub struct Casbin {
    enforcer: Mutex<Enforcer>,
}


#[allow(clippy::expect_used, clippy::unwrap_used)] // expect/unwrap justified for initialization
pub async fn init_casbin(database_url: String) -> Casbin {
    let model = DefaultModel::from_str(include_str!("rbac_model.conf")).await.unwrap();
    let adapter = SqlxAdapter::new(database_url, 10).await.unwrap();
    let enforcer = Mutex::new(Enforcer::new(model, adapter).await.unwrap());

    let casbin = Casbin {
        enforcer,
    };

    SiteRoles::create_roles_in_default_namespace(&casbin).await;

    casbin
}

impl Casbin {
    /// Example method to add a role to some subject (e.g., ("user:steve" -> "game:{id}:contributor") or "game:{id}:contributor").
    pub async fn add_subj_role(
        &self,
        subj: impl Into<DarnSubject>,
        role: impl Into<DarnRole>,
    ) {
        let subj = subj.into();
        let role = role.into();
        if let Err(e) = self.enforcer.lock().await.add_grouping_policy(vec![subj.to_string(), role.to_string()]).await {
            // only error when there's not a duplicate
            if !e.to_string().contains("already exists") {
                error!("error adding role to subject: {:?}", e);
            }
        }
    }

    /// Example method to remove a role from a subject.
    pub async fn remove_subj_role(
        &self,
        subj: impl Into<DarnSubject>,
        role: impl Into<DarnRole>,
    ) {
        let subj = subj.into();
        let role = role.into();
        let mut guard = self.enforcer.lock().await;
        if let Err(e) = guard.delete_role_for_user(&subj.to_string(), &role.to_string(), None).await {
            // TODO:
            error!("error removing role from subject: {:?}", e);
        }
    }

    /// Add a policy that allows `role` to do `action` on `object`.
    /// e.g. ("game:abc:author", "*", "game:abc").
    pub async fn add_allow_policy(
        &self,
        role: &DarnRole,
        action: impl PermissionMarker,
        object: impl Into<Darn>,
    ) {
        let action = action.to_string();
        let object = object.into().to_string();
        let mut guard = self.enforcer.lock().await;
        if let Err(e) = guard.add_policy(vec![role.to_string(), action, object, "allow".to_string()]).await {
            if !e.to_string().contains("already exists") {
                error!("error adding allow policy: {:?}", e);
            }
        }
    }

    /// Enforce that a `subject` can perform and `action` a `resource`.
    pub async fn enforce_action(
        &self,
        subj: impl Into<DarnSubject>,
        action: impl PermissionMarker,
        resource: impl Into<Darn>,
    ) -> bool {
        let subj = subj.into();
        let action = &action.to_string();
        let resource = resource.into();
        let guard = self.enforcer.lock().await;
        let result = guard.enforce((&subj.to_string(), action, &resource.to_string()));
        if let Err(err) = &result {
            warn!("Failed to enforce permissions for {}. {}: {}\n returning false", subj, action, err);
        }
        result.unwrap_or(false)
    }

    /// Retrieve all roles **directly** assigned to this subject (no inheritance).
    /// Example: If user "alice" has "admin" role, returns ["admin", ...].
    pub async fn get_explicit_roles(&self, subj: impl Into<DarnSubject>,) -> Vec<String> {
        let mut guard = self.enforcer.lock().await;
        guard.get_roles_for_user(&subj.into().to_string(), None)
    }

    /// Retrieve all **implicit** roles for a subject, including inherited roles.
    /// If "admin" inherits from "manager", and user "alice" has "admin",
    /// this returns ["admin", "manager"].
    pub async fn get_implicit_roles(&self, subj: impl Into<DarnSubject>,) -> Vec<String> {
        let mut guard = self.enforcer.lock().await;
        // `get_implicit_roles_for_user` can return an error; we just default to empty on error.
        guard.get_implicit_roles_for_user(&subj.into().to_string(), None)
    }

    /// Retrieve all **implicit** permissions for a subject. That includes direct
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
