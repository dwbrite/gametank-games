use std::fmt::Display;
use keycloak::types::Permission;
use maplit::hashmap;
use strum::IntoEnumIterator;
use strum_macros::{Display, ToString};
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use crate::auth::{Casbin, PermissionMarker, Permissions, RoleMarker, DefaultNamespace};
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

    // TODO: split this into two functions
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

impl DefaultNamespace for SiteRoles {
    fn default_namespace() -> Darn {
        Darn::new("site")
    }
}
