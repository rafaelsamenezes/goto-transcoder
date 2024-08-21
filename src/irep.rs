use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Irept {
    pub id: String,
    pub subt: Vec<Irept>,
    pub named_subt: HashMap<String, Irept>,
    pub comments: HashMap<String, Irept>,
}

impl std::hash::Hash for Irept {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        for irep in &self.subt {
            irep.hash(state);
        }
        for (name, irep) in &self.named_subt {
            name.hash(state);
            irep.hash(state);
        }
        for (name, irep) in &self.comments {
            name.hash(state);
            irep.hash(state);
        }
    }
}

impl PartialEq for Irept {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.subt == other.subt
            && self.named_subt == other.named_subt
            && self.comments == other.comments
    }
}
impl Eq for Irept {}

impl std::fmt::Display for Irept {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Irept: {}", self.id)
    }
}

impl Default for Irept {
    fn default() -> Self {
        Irept {
            id: String::from(""),
            subt: Vec::new(),
            named_subt: HashMap::new(),
            comments: HashMap::new(),
        }
    }
}
