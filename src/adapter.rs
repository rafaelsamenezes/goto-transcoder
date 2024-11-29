// This module is responsible to convert between CBMC <=> ESBMC

use std::collections::{HashMap, HashSet};

use crate::cbmc::{CBMCFunction, CBMCInstruction, CBMCParseResult, CBMCSymbol};
use crate::esbmc::ESBMCParseResult;
use crate::irep::Irept;

trait IrepAdapter {
    fn to_esbmc_irep(self) -> Irept;
}

impl From<CBMCParseResult> for ESBMCParseResult {
    fn from(data: CBMCParseResult) -> Self {
        let mut adapted = ESBMCParseResult {
            reader: (data.reader),
            symbols_irep: Vec::with_capacity(data.symbols_irep.len()),
            functions_irep: Vec::with_capacity(data.functions_irep.len()),
        };

        // First, we need to walk through the symbols and map all the
        // ref-types into concrete types

        let mut type_cache: HashMap<Irept, Irept> = HashMap::new();

        for mut sym in data.symbols_irep {
            if sym.is_type && sym.stype.id == "struct" {
                let tagname = Irept::from(format!("tag-{}", &sym.base_name));
                sym.stype.fix_type(&type_cache);
                type_cache.insert(tagname, sym.stype.clone());
            }
            adapted.symbols_irep.push(sym.to_esbmc_irep());
        }

        // Lets double check for fixes
        for symbol in &mut adapted.symbols_irep {
            symbol.fix_type(&type_cache);
            assert_ne!(symbol.named_subt["type"].id, "struct_tag");
            assert_ne!(symbol.named_subt["type"].id, "c_bool");
        }

        // NOTE: ESBMC/CBMC uses the number offset of the function as the target
        //       which is fine for most cases. But CBMC for some reason likes to
        //       start from 1 and have a target number associated to the instruction.
        //       So we first parse everything and then fix the target numbers
        for mut foo in data.functions_irep {
            let mut target_revmap: HashMap<u32, u32> = HashMap::new();

            for (index, inst) in &mut foo.instructions.iter().enumerate() {
                target_revmap.insert(inst.target_number, (index) as u32);
            }

            // lets fix the targets
            for f in &mut foo.instructions {
                for t in &mut f.targets {
                    let unsigned_value: u32 = t.id.parse().unwrap();
                    let target_fixed = target_revmap.get(&unsigned_value).unwrap().to_string();
                    t.id = target_fixed;
                }
            }

            let function_name = esbmcfixes::fix_name(&foo.name);
            let mut function_irep = foo.to_esbmc_irep();
            function_irep.fix_type(&type_cache);
            adapted.functions_irep.push((function_name, function_irep));
        }

        adapted
    }
}

mod esbmcfixes {
    use super::HashSet;
    use super::Irept;
    pub fn fix_name(name: &str) -> String {
        match name {
            "__CPROVER__start" => String::from("__ESBMC_main"),
            _ => String::from(name),
        }
    }

    pub fn fix_expression(irep: &mut Irept) {
        if irep.id == "side_effect" {
            irep.id = "sideeffect".to_string();
        }

        if irep.id == "constant" && irep.named_subt.contains_key("#base") {
            // Value ID might be the decimal/hexa representation, we want the binary one!
            let number = u64::from_str_radix(&irep.named_subt["value"].id, 16).unwrap();
            irep.named_subt.insert(
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

        if expressions.contains(&irep.id) {
            let mut operands = Irept::default();
            operands.subt = irep.subt.clone();
            irep.named_subt.insert("operands".to_string(), operands);
            irep.subt.clear();
        }

        for sub in &mut irep.subt {
            fix_expression(sub)
        }

        for (k, v) in &mut irep.named_subt {
            if k == "components" {
                for sub in &mut v.subt {
                    sub.id = "component".to_string();
                }
            }
            fix_expression(v);
        }
    }
}

impl IrepAdapter for CBMCInstruction {
    fn to_esbmc_irep(self) -> Irept {
        let mut result = Irept::default();

        // In ESBMC code arguments are expected to be inside the "operands"
        let mut code = self.code;
        let mut operands = Irept::default();
        operands.subt = code.subt.clone();
        code.subt.clear();
        code.named_subt.insert("operands".to_string(), operands);

        // Some checks
        if code.id != "nil" && code.named_subt.get("statement").unwrap().id == "assign" {
            assert_eq!(2, code.named_subt.get("operands").unwrap().subt.len());
        }
        result.named_subt.insert("code".to_string(), code);

        result
            .named_subt
            .insert("location".to_string(), self.source_location);
        result.named_subt.insert(
            "typeid".to_string(),
            Irept::from(self.instr_type.to_string()),
        );
        result.named_subt.insert("guard".to_string(), self.guard);

        if self.targets.len() != 0 {
            let mut t_ireps = Irept::default();
            for target in self.targets {
                t_ireps.subt.push(target);
            }
            result.named_subt.insert("targets".to_string(), t_ireps);
        }

        if self.labels.len() != 0 {
            let mut l_ireps = Irept::default();
            for label in self.labels {
                l_ireps.subt.push(Irept::from(label));
            }
            result.named_subt.insert("labels".to_string(), l_ireps);
        }

        // ESBMC stuff...
        result
            .named_subt
            .insert("function".to_string(), self.function);

        esbmcfixes::fix_expression(&mut result);
        result
    }
}

impl IrepAdapter for CBMCFunction {
    fn to_esbmc_irep(self) -> Irept {
        let mut result = Irept::from("goto-program");
        for instr in self.instructions {
            if instr.code.id == "nil" || instr.code.named_subt["statement"].id != "output" {
                result.subt.push(instr.to_esbmc_irep());
            }
        }
        result
    }
}

impl IrepAdapter for CBMCSymbol {
    fn to_esbmc_irep(self) -> Irept {
        let mut result = Irept::default();
        //result.id = String::from("symbol");
        result.named_subt.insert("type".to_string(), self.stype);
        result.named_subt.insert("symvalue".to_string(), self.value);
        result
            .named_subt
            .insert("location".to_string(), self.location);

        result
            .named_subt
            .insert("module".to_string(), Irept::from(&self.module));

        result
            .named_subt
            .insert("mode".to_string(), Irept::from(&self.mode));

        let name = match self.name.as_str() {
            "__CPROVER__start" => "__ESBMC_main".to_string(),
            _ => self.name.clone(),
        };

        let basename = match self.base_name.as_str() {
            "__CPROVER__start" => "__ESBMC_main".to_string(),
            _ => self.base_name.clone(),
        };

        if self.is_type {
            result
                .named_subt
                .insert("is_type".to_string(), Irept::from("1"));
        }

        result
            .named_subt
            .insert("base_name".to_string(), Irept::from(basename));

        result
            .named_subt
            .insert("name".to_string(), Irept::from(name));

        // Fix flags
        esbmcfixes::fix_expression(&mut result);
        result
    }
}

#[derive(Clone, Debug)]
enum Component {
    Struct {
        components: Vec<(String, Component)>,
    },
    Unsigned {
        width: usize,
    },
    Signed {
        width: usize,
    },
    Void,
    Pointer {
        to: Box<Component>,
    },
}

impl Component {}

fn from_struct(components: Vec<(String, Component)>) -> Irept {
    let mut result = Irept::from("struct");
    let mut subt: Irept = Irept::from("components");
    for (name, component) in components {
        let mut irep = Irept::from("component");
        irep.named_subt
            .insert("name".to_string(), Irept::from(name.clone()));
        irep.named_subt
            .insert("prettyname".to_string(), Irept::from(name));
        irep.named_subt
            .insert("type".to_string(), Irept::from(component));
        subt.subt.push(irep);
    }
    result.named_subt.insert("components".to_string(), subt);

    result
}

fn from_unsigned(width: usize) -> Irept {
    let mut result = Irept::from("unsignedbv");
    result
        .named_subt
        .insert("width".to_string(), Irept::from(width.to_string()));
    result
}

fn from_signed(width: usize) -> Irept {
    let mut result = Irept::from("signedbv");
    result
        .named_subt
        .insert("width".to_string(), Irept::from(width.to_string()));
    result
}

fn from_pointer(to: Box<Component>) -> Irept {
    let mut result = Irept::from("pointer");
    result
        .named_subt
        .insert("subtype".to_string(), Irept::from(*to));
    result
}

impl From<Component> for Irept {
    fn from(data: Component) -> Self {
        match data {
            Component::Struct { components } => from_struct(components),
            Component::Unsigned { width } => from_unsigned(width),
            Component::Signed { width } => from_signed(width),
            Component::Void => Irept::from("empty"),
            Component::Pointer { to } => from_pointer(to),
        }
    }
}

#[derive(Clone, Debug)]
struct Anon2Struct {
    bytes: Vec<u8>,
    counter: usize,
    cache: HashMap<String, Component>,
}

impl Anon2Struct {
    // Basic LL(k) parser.
    fn parse_component(&mut self) -> Component {
        assert!(self.counter + 3 <= self.bytes.len());
        if &self.bytes[self.counter..self.counter + 3] == "ST[".as_bytes() {
            self.counter = self.counter + 3;
            return self.parse_struct();
        } else if &self.bytes[self.counter..self.counter + 3] == "SYM".as_bytes() {
            self.counter = self.counter + 3;
            return self.parse_sym();
        } else if &self.bytes[self.counter..self.counter + 1] == "S".as_bytes() {
            self.counter = self.counter + 1;
            return self.parse_signed();
        } else if &self.bytes[self.counter..self.counter + 1] == "U".as_bytes() {
            self.counter = self.counter + 1;
            return self.parse_unsigned();
        } else if &self.bytes[self.counter..self.counter + 1] == "V".as_bytes() {
            self.counter = self.counter + 1;
            return Component::Void;
        } else if &self.bytes[self.counter..self.counter + 2] == "*{".as_bytes() {
            self.counter = self.counter + 2;
            return self.parse_pointer();
        }
        panic!("Missing something?");
    }

    fn parse_pointer(&mut self) -> Component {
        let component = self.parse_component();
        assert!(&self.bytes[self.counter..self.counter + 1] == "}".as_bytes());
        self.counter = self.counter + 1;
        Component::Pointer {
            to: Box::from(component),
        }
    }

    fn parse_unsigned(&mut self) -> Component {
        let mut id: Vec<u8> = Vec::new();
        let _ = loop {
            let char = &self.bytes[self.counter..self.counter + 1];

            self.counter = self.counter + 1;
            if char == "'".as_bytes() {
                self.counter = self.counter - 1;
                break;
            }
            id.push(char[0]);
        };

        let identifier = String::from_utf8_lossy(&id).to_string();
        let width: usize = identifier.as_str().parse().unwrap();
        Component::Unsigned { width }
    }

    fn parse_signed(&mut self) -> Component {
        let mut id: Vec<u8> = Vec::new();
        let _ = loop {
            let char = &self.bytes[self.counter..self.counter + 1];

            self.counter = self.counter + 1;
            if char == "'".as_bytes() {
                self.counter = self.counter - 1;
                break;
            }
            id.push(char[0]);
        };

        let identifier = String::from_utf8_lossy(&id).to_string();
        let width: usize = identifier.as_str().parse().unwrap();
        Component::Signed { width }
    }

    fn parse_name(&mut self) -> String {
        self.counter = self.counter + 1;
        let mut id: Vec<u8> = Vec::new();
        let _ = loop {
            let char = &self.bytes[self.counter..self.counter + 1];

            self.counter = self.counter + 1;
            if char == "'".as_bytes() {
                break;
            }
            id.push(char[0]);
        };

        String::from_utf8_lossy(&id).to_string()
    }

    fn parse_struct(&mut self) -> Component {
        let mut components: Vec<(String, Component)> = Vec::new();
        let _ = loop {
            let char = &self.bytes[self.counter..self.counter + 1];
            if char == "]".as_bytes() {
                self.counter = self.counter + 1;
                break;
            } else if char == "|".as_bytes() {
                self.counter = self.counter + 1;
            };

            let component = self.parse_component();
            assert!(&self.bytes[self.counter..self.counter + 1] == "'".as_bytes());
            let name = self.parse_name();
            components.push((name, component));
        };

        Component::Struct { components }
    }

    fn parse_sym(&mut self) -> Component {
        let mut id: Vec<u8> = Vec::new();
        let result = loop {
            let char = &self.bytes[self.counter..self.counter + 1];

            self.counter = self.counter + 1;
            if char == "'".as_bytes() || char == "}".as_bytes() {
                self.counter = self.counter - 1;
                break false;
            }
            if char == "=".as_bytes() {
                self.counter = self.counter + 1;
                break true;
            }
            id.push(char[0]);
        };

        let identifier = String::from_utf8_lossy(&id).to_string();
        if result {
            let component = self.parse_component();
            self.counter = self.counter + 1;
            self.cache.insert(identifier, component.clone());
            return component;
        }

        return self.cache[&identifier].clone();
    }
}

impl Irept {
    pub fn expand_anon_struct(&mut self) {
        if self.named_subt.contains_key("components") {
            return;
        }
        // ESBMC has no parser for this anon naming conventions.
        let identifier = self.named_subt["identifier"].id.as_bytes();
        assert!(&identifier[0..10] == "tag-#anon#".as_bytes());

        let mut parser = Anon2Struct {
            bytes: Vec::from(identifier),
            counter: 10,
            cache: HashMap::new(),
        };
        let parsed_struct = Irept::from(parser.parse_component());
        let components = parsed_struct.named_subt.get("components").unwrap().clone();
        self.named_subt.insert("components".to_string(), components);

        self.id = "struct".to_string();
    }

    pub fn fix_struct(&mut self) {
        self.id = "component".to_string();
    }

    pub fn fix_type(&mut self, cache: &HashMap<Irept, Irept>) {
        //

        if self.id == "c_bool" {
            self.id = String::from("signedbv");
            return;
        }

        if self.named_subt.contains_key("components") {
            for v in &mut self.named_subt.get_mut("components").unwrap().subt {
                v.fix_struct();
            }
        }

        if self.id == "pointer" && !self.named_subt.contains_key("subtype") {
            let mut operands = Irept::default();
            operands.subt = self.subt.clone();
            self.named_subt.insert("subtype".to_string(), operands);
            self.subt.clear();
        }

        if self.id == "array" && !self.named_subt.contains_key("subtype") && self.subt.len() > 0 {
            let magic = self.subt[0].clone();
            self.named_subt.insert("subtype".to_string(), magic);
            self.subt.clear();
            for (k, v) in &mut self.named_subt {
                if k == "size" {
                    if v.named_subt.contains_key("value") {
                        //v.fix_expression();
                        let number = u64::from_str_radix(&v.named_subt["value"].id, 16).unwrap();
                        v.named_subt.insert(
                            String::from("value"),
                            Irept::from(format!("{:064b}", number)),
                        );
                    }
                }
            }
        }

        if self.id != "struct_tag" {
            for v in &mut self.subt {
                v.fix_type(cache);
            }

            for (_, v) in &mut self.named_subt {
                v.fix_type(cache);
            }

            for (_, v) in &mut self.comments {
                v.fix_type(cache);
            }

            return;
        }

        if !self.named_subt.contains_key("identifier") {
            return;
        }

        if !cache.contains_key(&self.named_subt["identifier"]) {
            //self.expand_anon_struct();
            //self.fix_type(cache);
            return;
        }

        *self = cache[&self.named_subt["identifier"]].clone();
    }
}
