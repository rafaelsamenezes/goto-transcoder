use json::object;
use json::JsonValue;
use log::info;
use std::collections::HashMap;
use std::collections::HashSet;
#[derive(Clone, Debug)]
pub struct Irept {
    pub id: String,
    pub subt: Vec<Irept>,
    pub named_subt: HashMap<String, Irept>,
    pub comments: HashMap<String, Irept>,
}

impl Irept {
    pub fn get_nil() -> Self {
        Irept::from("nil")
    }
}

impl From<&Irept> for JsonValue {
    fn from(data: &Irept) -> Self {
        let mut obj = object! {id: data.id.clone()};

        let mut sub_vec: Vec<JsonValue> = Vec::new();
        for sub in &data.subt {
            sub_vec.push(JsonValue::from(sub));
        }
        if sub_vec.len() > 0 {
            obj["subt"] = JsonValue::from(sub_vec);
        }

        for (k, v) in &data.named_subt {
            obj[k] = JsonValue::from(v);
        }

        for (k, v) in &data.comments {
            obj[k] = JsonValue::from(v);
        }
        obj
    }
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
        let as_json = JsonValue::from(self);
        write!(f, "{}", json::stringify_pretty(as_json, 4))
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

impl From<&String> for Irept {
    fn from(data: &String) -> Self {
        let mut res = Irept::default();
        res.id = data.clone();
        res
    }
}

impl From<String> for Irept {
    fn from(data: String) -> Self {
        let mut res = Irept::default();
        res.id = data;
        res
    }
}

impl From<&str> for Irept {
    fn from(data: &str) -> Self {
        let mut res = Irept::default();
        res.id = data.to_string();
        res
    }
}

impl Irept {
    pub fn fix_expression(&mut self) {
        if self.id == "side_effect" {
            self.id = "sideeffect".to_string();
        }

        if self.id == "constant" && self.named_subt.contains_key("#base") {
            // Value ID might be the decimal/hexa representation, we want the binary one!
            let number = u64::from_str_radix(&self.named_subt["value"].id, 16).unwrap();
            self.named_subt.insert(
                String::from("value"),
                Irept::from(format!("{:064b}", number)),
            );
        }

        let expressions: HashSet<String> = HashSet::from(
            [
                "if",
                "member",
                "typecast",
                "notequal",
                "and",
                "or",
                "mod",
                "not",
                "*",
                "/",
                "+",
                "-",
                "=",
                "<",
                ">",
                "lshr",
                "shl",
                "address_of",
                "index",
                "byte_extract_little_endian",
                "pointer_object",
                "array_of",
                "sideeffect",
                "dereference",
                "bitand",
            ]
            .map(|x| x.to_string()),
        );

        if expressions.contains(&self.id) {
            let mut operands = Irept::default();
            operands.subt = self.subt.clone();
            self.named_subt.insert("operands".to_string(), operands);
            self.subt.clear();
        }

        for sub in &mut self.subt {
            sub.fix_expression();
        }

        for (k, v) in &mut self.named_subt {
            if k == "components" {
                for sub in &mut v.subt {
                    sub.id = "component".to_string();
                }
            }
            v.fix_expression();
        }
    }
}
