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
            .insert("name".to_string(), Irept::from(&data.name));
        result
            .named_subt
            .insert("module".to_string(), Irept::from(&data.module));
        result
            .named_subt
            .insert("base_name".to_string(), Irept::from(&data.base_name));
        result
            .named_subt
            .insert("mode".to_string(), Irept::from(&data.mode));

        // Fix flags
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

#[derive(Clone, Debug)]
pub struct CBMCParser {
    pub reader: ByteReader,
    pub symbols_irep: Vec<Irept>,
    pub functions_irep: Vec<(String, Irept)>,
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

pub fn process_cbmc_file(path: &str) -> CBMCParser {
    let mut result = CBMCParser {
        reader: ByteReader::read_file(path),
        functions_irep: Vec::new(),
        symbols_irep: Vec::new(),
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

        let mut symname = result.reader.read_cbmc_string_ref();
        if symname == "__CPROVER__start" {
            symname = "__ESBMC_main".to_string();
        }
        sym.name = symname;
        sym.module = result.reader.read_cbmc_string_ref();

        let mut symbasename = result.reader.read_cbmc_string_ref();
        if symbasename == "__CPROVER__start" {
            symbasename = "__ESBMC_main".to_string();
        }
        debug!("Basename: {}", symbasename);
        sym.base_name = symbasename;
        sym.mode = result.reader.read_cbmc_string_ref();

        sym.pretty_name = result.reader.read_cbmc_string_ref();

        result.reader.read_cbmc_word();
        sym.flags = result.reader.read_cbmc_word();
        debug!("Symbol name {}", sym.name);
        result.symbols_irep.push(Irept::from(sym));
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
        result
            .functions_irep
            .push((function.name.clone(), Irept::from(function)));
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
        ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, "/tmp/test_cbmc.goto");
    }
}
