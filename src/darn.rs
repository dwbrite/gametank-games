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
    
    pub fn from_parent(parent: &Darn, name: String) -> Darn {
        Darn { parent: Some(Box::new(parent.clone())), name }
    }

    pub fn full_name(&self) -> String {
        let mut full_name = String::new();
        if let Some(d) = &self.parent {
            full_name.push_str(&d.full_name());
        }
        full_name.push_str(&self.name);
        full_name
    }
}
