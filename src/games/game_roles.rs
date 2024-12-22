// use std::fmt::Display;
// use strum_macros::{Display, ToString};
// use tracing_subscriber::fmt::format;
// use uuid::Uuid;
// use crate::auth::{apply_role_policies, Casbin, HasPermissions, PermissionsInfo, RoleMarker};
// use crate::darn::Darn;
// use crate::games::game_roles::GamePermissions::*;
// use crate::games::game_roles::GameRoles::{Collaborator, Contributor};
//
// #[derive(Display)]
// #[strum(serialize_all = "snake_case")]
// pub enum GameRoles {
//     Author,
//     Collaborator,
//     Contributor,
//     Previewer,
// }
//
// impl RoleMarker<GamePermissions, GameRoles> for GameRoles {}
//
// #[derive(Display)]
// #[strum(serialize_all = "snake_case")]
// pub enum GamePermissions {
//     #[strum(serialize = "*")]
//     All,
//     ModifyCollaborators,
//     ModifyContributors,
//     ModifyPreviewers,
//     Patch,
//     Delete,
//     View,
// }
//
//
//
// impl HasPermissions<GamePermissions, GameRoles> for GameRoles {
//     fn permissions(&self) -> PermissionsInfo<GamePermissions, GameRoles> {
//         match self {
//             GameRoles::Author => vec![ All ],
//             GameRoles::Collaborator => vec![
//                 ModifyCollaborators,
//                 ModifyContributors,
//                 ModifyPreviewers,
//             ],
//             GameRoles::Contributor => vec![
//                 Patch,
//                 View,
//             ],
//             GameRoles::Previewer => vec![
//                 View,
//             ],
//         }
//     }
// }
//
// pub async fn add_game_roles<D: Into<Darn>>(casbin: &Casbin, game_darn: D) -> &'static [GameRoles] {
//     let roles = &[
//         GameRoles::Author,
//         GameRoles::Collaborator,
//         GameRoles::Contributor,
//         GameRoles::Previewer,
//     ];
//
//     let game_darn = game_darn.into();
//
//     Self::add_object_roles();
//
//     apply_role_policies(casbin, &game_darn, roles).await;
//     casbin.add_subj_role(&Collaborator.to_darn_role(&game_darn), &Contributor.to_darn_role(&game_darn)).await;
//
//     roles
// }
