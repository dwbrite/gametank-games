use std::collections::HashMap;
use maplit::hashmap;
use strum_macros::Display;
use crate::auth::{PermissionMarker, RoleMarker, DefaultNamespace};
use crate::auth::site_roles::SitePermissions::*;
use crate::auth::site_roles::SiteRoles::*;
use crate::darn::Darn;

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
