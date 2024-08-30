use std::collections::HashMap;

pub use crate::Irept;
use crate::{ByteReader, ByteWriter};
use log::{debug, error, trace};

#[derive(Clone, Debug)]
pub struct CBMCSymbol {
    pub stype: Irept,
    pub value: Irept,
    pub location: Irept,
    pub name: String,
    pub module: String,
    pub base_name: String,
    pub mode: String,
    pub pretty_name: String,
    pub flags: u32,
    pub is_type: bool
}

impl Default for CBMCSymbol {
    fn default() -> Self {
        CBMCSymbol {
            stype: Irept::default(),
            value: Irept::default(),
            location: Irept::default(),
            name: String::default(),
            module: String::default(),
            base_name: String::default(),
            mode: String::default(),
            pretty_name: String::default(),
            flags: 0,
            is_type: false
        }
    }
}

impl From<CBMCSymbol> for Irept {
    fn from(data: CBMCSymbol) -> Self {
        let mut result = Irept::default();
        //result.id = String::from("symbol");
        result.named_subt.insert("type".to_string(), data.stype);
        result.named_subt.insert("symvalue".to_string(), data.value);
        result
            .named_subt
            .insert("location".to_string(), data.location);

        result
            .named_subt
            .insert("module".to_string(), Irept::from(&data.module));

        result
            .named_subt
            .insert("mode".to_string(), Irept::from(&data.mode));

        let name = match data.name.as_str() {
            "__CPROVER__start" => "__ESBMC_main".to_string(),
            _ => data.name.clone(),
        };

        let basename = match data.base_name.as_str() {
            "__CPROVER__start" => "__ESBMC_main".to_string(),
            _ => data.base_name.clone(),
        };

        if data.is_type {
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
        result.fix_expression();
        result
    }
}

#[derive(Clone, Debug)]
pub struct CBMCInstruction {
    pub code: Irept,
    pub source_location: Irept,
    pub instr_type: u32,
    pub guard: Irept,
    pub target_number: u32,
    pub targets: Vec<Irept>,
    pub labels: Vec<String>,
    pub function: Irept,
}

#[derive(Clone, Debug)]
pub struct CBMCFunction {
    pub name: String,
    pub instructions: Vec<CBMCInstruction>,
}

impl From<CBMCInstruction> for Irept {
    fn from(data: CBMCInstruction) -> Self {
        let mut result = Irept::default();

        // In ESBMC code arguments are expected to be inside the "operands"
        let mut code = data.code;
        let mut operands = Irept::default();
        operands.subt = code.subt.clone();
        code.subt.clear();
        code.named_subt.insert("operands".to_string(), operands);

        result.named_subt.insert("code".to_string(), code);

        result
            .named_subt
            .insert("location".to_string(), data.source_location);
        result.named_subt.insert(
            "typeid".to_string(),
            Irept::from(data.instr_type.to_string()),
        );
        result.named_subt.insert("guard".to_string(), data.guard);

        if data.targets.len() != 0 {
            let mut t_ireps = Irept::default();
            for target in data.targets {
                t_ireps.subt.push(target);
            }
            result.named_subt.insert("targets".to_string(), t_ireps);
        }

        if data.labels.len() != 0 {
            let mut l_ireps = Irept::default();
            for label in data.labels {
                l_ireps.subt.push(Irept::from(label));
            }
            result.named_subt.insert("labels".to_string(), l_ireps);
        }

        // ESBMC stuff...
        result
            .named_subt
            .insert("function".to_string(), data.function);

        result.fix_expression();
        result
    }
}

impl From<CBMCFunction> for Irept {
    fn from(data: CBMCFunction) -> Self {
        let mut result = Irept::from("goto-program");
        for instr in data.instructions {
            if instr.code.id == "nil" || instr.code.named_subt["statement"].id != "output" {
                result.subt.push(Irept::from(instr));
            }
        }
        result
    }
}

#[derive(Clone, Debug)]
pub struct CBMCParser {
    pub reader: ByteReader,
    pub symbols_irep: Vec<Irept>,
    pub functions_irep: Vec<(String, Irept)>,
    pub struct_cache: HashMap<Irept, Irept>,
}

impl Irept {
    pub fn fix_type(&mut self, cache: &HashMap<Irept, Irept>) {
        if self.id != "struct_tag" {
            for v in &mut self.subt {
                v.fix_type(cache);
            }

            for (_,v) in &mut self.named_subt {
                v.fix_type(cache);
            }

            for (_,v) in &mut self.comments {
                v.fix_type(cache);
            }
            
            return;
        }
      
        if !self.named_subt.contains_key("identifier") {
            return;
        }


        if !cache.contains_key(&self.named_subt["identifier"]) {
            panic!("Cache miss {}", self);
        }

        *self = cache[&self.named_subt["identifier"]].clone();
    }
}

pub fn process_cbmc_file(path: &str) -> CBMCParser {
    let mut result = CBMCParser {
        reader: ByteReader::read_file(path),
        functions_irep: Vec::new(),
        symbols_irep: Vec::new(),
        struct_cache: HashMap::new(),
    };

    result.reader.check_cbmc_header().unwrap();
    result.reader.check_cbmc_version().unwrap();

    // Symbol table
    let number_of_symbols = result.reader.read_cbmc_word();
    debug!("Got {} symbols", number_of_symbols);

    for _ in 0..number_of_symbols {
        let mut sym = CBMCSymbol::default();
        sym.stype = result.reader.read_cbmc_reference();
        sym.value = result.reader.read_cbmc_reference();
        sym.location = result.reader.read_cbmc_reference();

        sym.name = result.reader.read_cbmc_string_ref();
        sym.module = result.reader.read_cbmc_string_ref();

        sym.base_name = result.reader.read_cbmc_string_ref();
        sym.mode = result.reader.read_cbmc_string_ref();

        sym.pretty_name = result.reader.read_cbmc_string_ref();

        result.reader.read_cbmc_word();
        sym.flags = result.reader.read_cbmc_word();

        sym.is_type = sym.flags & (1 << 15) != 0;
        if sym.is_type && sym.stype.id == "struct" {
            // Type caching
            sym.stype.named_subt.insert("tag".to_string(), Irept::from(&sym.base_name));
            let tagname = Irept::from(format!("tag-{}", &sym.base_name));
            result.struct_cache.insert(tagname, sym.stype.clone());
        }
        
        // sym.is_weak = (flags &(1 << 16))!=0;
        // sym.is_type = (flags &(1 << 15))!=0;
        // sym.is_property = (flags &(1 << 14))!=0;
        // sym.is_macro = (flags &(1 << 13))!=0;
        // sym.is_exported = (flags &(1 << 12))!=0;
        // // sym.is_input = (flags &(1 << 11))!=0;
        // sym.is_output = (flags &(1 << 10))!=0;
        // sym.is_state_var = (flags &(1 << 9))!=0;
        // sym.is_parameter = (flags &(1 << 8))!=0;
        // sym.is_auxiliary = (flags &(1 << 7))!=0;
        // // sym.binding = (flags &(1 << 6))!=0;
        // sym.is_lvalue = (flags &(1 << 5))!=0;
        // sym.is_static_lifetime = (flags &(1 << 4))!=0;
        // sym.is_thread_local = (flags &(1 << 3))!=0;
        // sym.is_file_local = (flags &(1 << 2))!=0;
        // sym.is_extern = (flags &(1 << 1))!=0;
        // sym.is_volatile = (flags &1)!=0;

        let mut symbol_irep = Irept::from(sym);
        symbol_irep.fix_type(&result.struct_cache);
        result.symbols_irep.push(symbol_irep);
    }

    // Functions
    let number_of_functions = result.reader.read_cbmc_word();
    debug!("Got {} functions", number_of_functions);
    for _ in 0..number_of_functions {
        let mut function_name = result.reader.read_gb_string();
        if function_name == "__CPROVER__start" {
            function_name = "__ESBMC_main".to_string();
        }
        let mut function = CBMCFunction {
            name: function_name,
            instructions: Vec::new(),
        };

        for _ in 0..result.reader.read_cbmc_word() {
            // # instructions
            let code = result.reader.read_cbmc_reference();

            let source_location = result.reader.read_cbmc_reference();
            let instr_type = result.reader.read_cbmc_word();
            let guard = result.reader.read_cbmc_reference();
            let target_number = result.reader.read_cbmc_word();

            // Add targets
            let t_count = result.reader.read_cbmc_word();
            let mut targets: Vec<Irept> = Vec::new();
            for _ in 0..t_count {
                targets.push(Irept::from(result.reader.read_cbmc_word().to_string()));
            }

            // Add labels
            let l_count = result.reader.read_cbmc_word();
            let mut labels: Vec<String> = Vec::default();
            for _ in 0..l_count {
                let label = result.reader.read_cbmc_string_ref();
                labels.push(label);
            }

            function.instructions.push(CBMCInstruction {
                code,
                source_location,
                instr_type,
                guard,
                target_number,
                targets,
                labels,
                function: Irept::from(&function.name),
            })
        }
        let function_name = function.name.clone();
        let mut function_irep = Irept::from(function);
        function_irep.fix_type(&result.struct_cache);
        result
            .functions_irep
            .push((function_name, function_irep));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sql::SqlWriter;
    #[test]
    fn test_cbmc_to_sqlite_file() {
        let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(err) => panic!("Could not open cargo folder. {}", err),
        };
        let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello-gb.goto");
        assert!(test_path.exists());

        let result = process_cbmc_file(test_path.to_str().unwrap());

        std::fs::remove_file("/tmp/test_cbmc.sqlite3").ok();
        SqlWriter::write_to_file(
            result.symbols_irep.clone(),
            result.functions_irep.clone(),
            "/tmp/test_cbmc.sqlite3",
        );
    }

    #[test]
    fn test_cbmc_to_esbmc_file() {
        let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(err) => panic!("Could not open cargo folder. {}", err),
        };
        let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello-gb.goto");
        assert!(test_path.exists());

        let result = crate::cbmc::process_cbmc_file(test_path.to_str().unwrap());

        std::fs::remove_file("/tmp/test_cbmc.goto").ok();
        ByteWriter::write_to_file(
            result.symbols_irep,
            result.functions_irep,
            "/tmp/test_cbmc.goto",
        );
    }
}
