use std::fmt::Display;
use strum_macros::{Display, ToString};
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use crate::auth::{apply_role_policies, Casbin, HasPermissions, IntoDarnWithContext};
use crate::darn::Darn;
use crate::games::game_roles::GamePermissions::*;
use crate::games::game_roles::GameRoles::{Collaborator, Contributor};

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum GameRoles {
    Author,
    Collaborator,
    Contributor,
    Previewer,
}

impl IntoDarnWithContext for GameRoles {}


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



impl HasPermissions<GamePermissions> for GameRoles {
    fn permissions(&self) -> Vec<GamePermissions> {
        match self {
            GameRoles::Author => vec![ All ],
            GameRoles::Collaborator => vec![
                ModifyCollaborators,
                ModifyContributors,
                ModifyPreviewers,
            ],
            GameRoles::Contributor => vec![
                Patch,
                View,
            ],
            GameRoles::Previewer => vec![
                View,
            ],
        }
    }
}

pub async fn add_game_roles(casbin: &Casbin, game_darn: &Darn) -> &'static [GameRoles] {
    let roles = &[
        GameRoles::Author,
        GameRoles::Collaborator,
        GameRoles::Contributor,
        GameRoles::Previewer,
    ];

    apply_role_policies(casbin, game_darn, roles).await;
    casbin.add_role_subset(Collaborator.to_darn(game_darn), Contributor.to_darn(game_darn)).await;

    roles
}
