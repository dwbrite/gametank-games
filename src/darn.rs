use std::fmt::{Display, Formatter};

/// Devin's abstract resource names

#[derive(Debug, Clone)]
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
}

impl Display for Darn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_name())
    }
}
