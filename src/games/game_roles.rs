use maplit::hashmap;
use strum_macros::Display;
use crate::auth::{PermissionMarker, Permissions, RoleMarker};
use crate::games::game_roles::GameRoles::*;
use crate::games::game_roles::GamePermissions::*;

// use std::fmt::Display;
// use strum_macros::{Display, ToString};
// use tracing_subscriber::fmt::format;
// use uuid::Uuid;
// use crate::auth::{apply_role_policies, Casbin, HasPermissions, PermissionsInfo, RoleMarker};
// use crate::darn::Darn;
// use crate::games::game_roles::GamePermissions::*;
// use crate::games::game_roles::GameRoles::{Collaborator, Contributor};
//
#[derive(Display, Eq, PartialEq, Hash, Copy, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum GameRoles {
    Author,
    Collaborator,
    Contributor,
    Previewer,
}


#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum GamePermissions {
    #[strum(serialize = "*")]
    All,
    ModifyCollaborators,
    ModifyContributors,
    ModifyPreviewers,
    Patch,
    Delete,
    View,
}

impl PermissionMarker for GamePermissions {}

impl RoleMarker for GameRoles {
    type RolePermission = GamePermissions;

    fn permissions() -> Permissions<Self::RolePermission, Self> {
        let allowed_actions = hashmap!{
            Author => vec![All],
            Collaborator => vec![ModifyCollaborators, ModifyContributors],
            Contributor => vec![ModifyContributors, Patch, View],
            Previewer => vec![View],
        };

        let inheritance = vec![
            (Author, Collaborator),
            (Collaborator, Contributor),
        ];

        Permissions {
            allowed_actions,
            inheritance
        }
    }
}
