use std::fmt::Display;
use std::hash::Hash;
use log::warn;
use std::collections::HashMap;
use crate::auth::Casbin;
use crate::darn::{Darn, DarnRole};

pub trait PermissionMarker: Display + Copy {}

pub trait DefaultNamespace: RoleMarker {
    fn default_namespace() -> Darn;

    /// Converts the role into a DarnRole based on the provided context.
    fn to_darn_role(&self) -> DarnRole {
        self.to_darn_role_with_context(&Self::default_namespace())
    }

    async fn create_roles_in_default_namespace(casbin: &Casbin) {
        Self::create_roles_in_namespace(casbin, Self::default_namespace()).await;
    }
}

pub trait RoleMarker: Display + PartialEq + Eq + Hash + Sized + Copy + Clone {
    type RolePermission: PermissionMarker;

    /// Returns a list of permissions associated directly with the role.
    fn allowed_actions() -> HashMap<Self, Vec<Self::RolePermission>>;
    fn inheritance() -> Vec<(Self, Self)>;

    /// Converts the role into a DarnRole based on the provided context.
    fn to_darn_role_with_context(&self, ctx: &Darn) -> DarnRole {
        DarnRole::from_context(&self.to_string(), &ctx)
    }

    /// Adds roles to an object using Casbin.
    async fn create_roles_in_namespace(casbin: &Casbin, namespace: impl Into<Darn>) {
        let ns = &namespace.into();

        println!("swag");
        for (role, actions) in &Self::allowed_actions() {
            println!("found a role");
            let namespaced_role = &role.to_darn_role_with_context(ns);
            for action in actions {
                println!("should create this policy: {} {}", namespaced_role, action);
                match casbin.add_allow_policy(namespaced_role, *action, ns).await {
                    Ok(_) => {}
                    Err(_) => { warn!("Failed to apply casbin policy on role {}. (This role has likely been initialized already? Maybe I should delete the roles first??? sus.)", role); }
                }
            }
        }

        for (superset, subset) in &Self::inheritance() {
            let superset_role = superset.to_darn_role_with_context(ns);
            let subset_role = subset.to_darn_role_with_context(ns);
            match casbin.add_subj_role(superset_role, subset_role).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}