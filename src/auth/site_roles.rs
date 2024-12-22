use std::fmt::Display;
use strum_macros::{Display, ToString};
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use crate::auth::{apply_role_policies, Casbin, HasPermissions, IntoDarnWithContext};
use crate::auth::site_roles::SitePermissions::*;
use crate::auth::site_roles::SiteRoles::*;
use crate::darn::{DarNS, Darn};

pub const SITE_NS: DarNS = DarNS("site");

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
pub enum SiteRoles {
    Admin,
    SrModerator,
    Moderator,
    User,
    Guest
}

impl IntoDarnWithContext for SiteRoles {}

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


impl HasPermissions<SitePermissions> for SiteRoles {
    fn permissions(&self) -> Vec<SitePermissions> {
        match self {
            Admin => vec![ All ],
            SrModerator => vec![ AddModerator ],
            Moderator => vec![ BanGame, BanUser ],
            User => vec![ CreateGame, ViewPublic ],
            Guest => vec![ ViewPublic ],
        }
    }
}

pub async fn add_site_roles(casbin: &Casbin, site_darn: &Darn) -> &'static [SiteRoles] {
    let roles = &[
        Admin,
        SrModerator,
        Moderator,
        User,
        Guest
    ];

    apply_role_policies(casbin, site_darn, roles).await;
    casbin.add_subj_role(&SrModerator.to_darn(site_darn), &Moderator.to_darn(site_darn)).await;
    casbin.add_subj_role(&Moderator.to_darn(site_darn), &User.to_darn(site_darn)).await;

    roles
}
