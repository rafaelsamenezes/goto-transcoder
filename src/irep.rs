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


impl Irept {
    fn find(&self, id: &String) -> Irept {
        let result = match self.named_subt.get(id) {
            Some(v) => v.clone(),
            None => Irept::default(),
        };
        result
    }

    
    pub fn get_type(&self) -> Irept {
        self.find(&String::from("type"))
    }
    pub fn get_symvalue(&self) -> Irept {
        self.find(&String::from("symvalue"))
    }
    pub fn get_location(&self) -> Irept {
        self.find(&String::from("location"))
    }
    pub fn get_name(&self) -> String {
        self.find(&String::from("name")).id
    }
    pub fn get_module(&self) -> String {
        self.find(&String::from("module")).id
    }
    pub fn get_base_name(&self) -> String {
        self.find(&String::from("base_name")).id
    }
    pub fn get_mode(&self) -> String {
        self.find(&String::from("mode")).id
    }
    pub fn get_bool(&self, id: &String) -> bool {
        self.find(id).id != "0"
    }
    pub fn is_type(&self) -> bool {
        self.get_bool(&String::from("is_type"))
    }
    pub fn is_macro(&self) -> bool {
        self.get_bool(&String::from("is_macro"))
    }
    pub fn is_parameter(&self) -> bool {
        self.get_bool(&String::from("is_parameter"))
    }
    pub fn is_lvalue(&self) -> bool {
        self.get_bool(&String::from("is_lvalue"))
    }
    pub fn is_static_lifetime(&self) -> bool {
        self.get_bool(&String::from("static_lifetime"))
    }
    pub fn is_file_local(&self) -> bool {
        self.get_bool(&String::from("file_local"))
    }
    pub fn is_extern(&self) -> bool {
        self.get_bool(&String::from("is_extern"))
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

