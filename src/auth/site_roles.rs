use std::fmt::Display;
use keycloak::types::Permission;
use maplit::hashmap;
use strum::IntoEnumIterator;
use strum_macros::{Display, ToString};
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use crate::auth::{Casbin, PermissionMarker, Permissions, RoleMarker};
use crate::auth::site_roles::SitePermissions::*;
use crate::auth::site_roles::SiteRoles::*;
use crate::darn::{Darn, DarnRole};
// use crate::darn::{DarNS, Darn};

// pub const SITE_NS: DarNS = DarNS("site");

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum SiteRoles {
    Admin,
    SrModerator,
    Moderator,
    User,
    Guest
}

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum SitePermissions {
    #[strum(serialize = "*")]
    All,

    AddModerator,

    BanGame,
    BanUser,

    CreateGame,
    ViewPublic,
}

impl PermissionMarker for SitePermissions {}

impl RoleMarker for SiteRoles {
    type RolePermission = SitePermissions;

    fn permissions() -> Permissions<Self::RolePermission, Self> {
        let allowed_actions = hashmap!{
            Admin => vec![ All ],
            SrModerator => vec![ AddModerator ],
            Moderator => vec![ BanGame, BanUser ],
            User => vec![ CreateGame, ViewPublic ],
            Guest => vec![ ViewPublic ],
        };

        let inheritance = vec![
            (SrModerator, Moderator),
            (Moderator, User)
        ];

        Permissions {
            allowed_actions,
            inheritance
        }
    }
}

pub async fn add_resource_roles(casbin: &Casbin, resource_name: impl Into<Darn>, roles: Vec<impl Into<DarnRole>>, ) {

}

pub async fn add_site_roles(casbin: &Casbin, site_darn: impl Into<Darn>) {
    let roles: Vec<SiteRoles> = SiteRoles::iter().collect();
    let site_darn = &site_darn.into();
    SiteRoles::add_permissions_for_object(casbin, site_darn, roles);
}
