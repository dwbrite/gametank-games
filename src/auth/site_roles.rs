use std::collections::HashMap;
use std::fmt::Display;
use keycloak::types::Permission;
use maplit::hashmap;
use strum::IntoEnumIterator;
use strum_macros::{Display, ToString};
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use crate::auth::{Casbin, PermissionMarker, RoleMarker, DefaultNamespace};
use crate::auth::site_roles::SitePermissions::*;
use crate::auth::site_roles::SiteRoles::*;
use crate::darn::{Darn, DarnRole};

#[derive(Display, Eq, PartialEq, Hash, Copy, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum SiteRoles {
    Admin,
    SrModerator,
    Moderator,
    User,
    Guest
}

#[derive(Display, Copy, Clone)]
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

    fn allowed_actions() -> HashMap<Self, Vec<Self::RolePermission>> {
        hashmap!{
            Admin => vec![ All ],
            SrModerator => vec![ AddModerator ],
            Moderator => vec![ BanGame, BanUser ],
            User => vec![ CreateGame, ViewPublic ],
            Guest => vec![ ViewPublic ],
        }
    }

    fn inheritance() -> Vec<(Self, Self)> {
        vec![
            (SrModerator, Moderator),
            (Moderator, User)
        ]
    }
}

impl DefaultNamespace for SiteRoles {
    fn default_namespace() -> Darn {
        Darn::new("site")
    }
}
