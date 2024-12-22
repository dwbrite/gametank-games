use std::fmt::{Display, Formatter};
use crate::auth::IntoDarnWithContext;

/// Devin's abstract resource names

pub struct DarNS(pub &'static str);

impl DarNS {
    pub fn new_child(&self, name: &str) -> Darn {
        Darn::new(&self.0).new_child(name)
    }

    pub fn role(&self, role: &dyn IntoDarnWithContext) -> Darn {
        role.to_darn(&self.into())
    }
}

impl From<DarNS> for Darn {
    fn from(ns: DarNS) -> Self {
        Darn::new(&ns.0)
    }
}

impl From<&DarNS> for Darn {
    fn from(ns: &DarNS) -> Self {
        Darn::new(&ns.0)
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

    pub fn role(&self, role: &dyn IntoDarnWithContext) -> Darn {
        role.to_darn(self)
    }
}


impl Display for Darn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}
