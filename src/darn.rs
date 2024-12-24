use std::fmt::{Display, Formatter};
use std::ops::Deref;
use crate::auth::authn_keycloak::KeycloakUserInfo;

/// Devin's abstract resource names

#[derive(Debug, Clone)]
pub struct DarnRole(Darn);
impl DarnRole {
    pub fn from_context(role_name: &str, ctx: &Darn) -> DarnRole {
        let ns = Darn::new("role");
        let resource = ctx.new_child(role_name);
        DarnRole(ns.new_child(&resource.to_string()))
    }
}


impl From<&DarnUser> for Darn {
    fn from(user: &DarnUser) -> Self {
        user.0.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DarnUser(Darn);

impl Display for DarnUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

impl Display for DarnRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}

impl From<&KeycloakUserInfo> for DarnUser {
    fn from(user: &KeycloakUserInfo) -> Self {
        DarnUser(Darn::new("user").new_child(&user.sub))
    }
}

pub type DarnSubject = Darn;

impl From<DarnRole> for DarnSubject {
    fn from(role: DarnRole) -> Self {
        role.0.clone()
    }
}


impl From<DarnUser> for DarnSubject {
    fn from(user: DarnUser) -> Self {
        user.0.clone()
    }
}

impl Deref for DarnUser {
    type Target = Darn;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DarnRole {
    type Target = Darn;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Darn {
    name: String,
    parent: Option<Box<Darn>>,
}

impl Darn {
    pub fn new(name: &str) -> Darn {
        Darn {
            parent: None,
            name: name.to_string(),
        }
    }

    pub fn with_namespace(namespace: &str, name: &str) -> Darn {
        Self::new(namespace).new_child(name)
    }

    pub fn full_name(&self) -> String {
        let mut full_name = String::new();
        if let Some(d) = &self.parent {
            full_name.push_str(&format!("{}:", &d.full_name()));
        }
        full_name.push_str(&self.name);
        full_name
    }

    pub fn new_child(&self, name: &str) -> Darn {
        Darn { parent: Some(Box::new(self.clone())), name: name.to_string() }
    }
}

impl From<&Darn> for Darn {
    fn from(value: &Darn) -> Self {
        value.clone()
    }
}


impl Display for Darn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}
