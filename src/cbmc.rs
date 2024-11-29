use crate::bytereader::ByteReader;
pub use crate::Irept;
use log::debug;
use log::info;
use std::collections::HashMap;

///////////////
// CBMC DATA //
///////////////

// Direct parsing result of a symbol
#[derive(Clone, Debug)]
pub struct CBMCSymbol {
    pub stype: Irept, // stype => type. Rust reserves some weird words
    pub value: Irept,
    pub location: Irept,
    pub name: String,
    pub module: String,
    pub base_name: String,
    pub mode: String,
    pub pretty_name: String,
    pub flags: u32,
    pub is_type: bool,
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
            is_type: false,
        }
    }
}

// Direct parsing result of an instruction
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

// Direct parsing result of a function
#[derive(Clone, Debug)]
pub struct CBMCFunction {
    pub name: String,
    pub instructions: Vec<CBMCInstruction>,
}

////////////
// PARSER //
////////////

#[derive(Clone, Debug)]
pub struct CBMCParseResult {
    pub reader: ByteReader,
    pub symbols_irep: Vec<CBMCSymbol>,
    pub functions_irep: Vec<CBMCFunction>,
}

pub fn process_cbmc_file(path: &str) -> CBMCParseResult {
    let mut result = CBMCParseResult {
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
        // Type is a... type
        sym.stype = result.reader.read_cbmc_reference();
        // Value is an expr
        sym.value = result.reader.read_cbmc_reference();
        // Location is just a string
        sym.location = result.reader.read_cbmc_reference();
        // Name is just a string
        sym.name = result.reader.read_cbmc_string_ref();
        // Module is just a string
        sym.module = result.reader.read_cbmc_string_ref();
        // Base name is just a string
        sym.base_name = result.reader.read_cbmc_string_ref();
        // Symbol mode conveys the language (C, C++, Rust, etc)
        sym.mode = result.reader.read_cbmc_string_ref();
        // String
        sym.pretty_name = result.reader.read_cbmc_string_ref();

        // Ordering is used for historical reasons.
        let ordering = result.reader.read_cbmc_word();
        assert_eq!(ordering, 0);

        sym.flags = result.reader.read_cbmc_word();

        sym.is_type = sym.flags & (1 << 15) != 0;

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
        // sym.is_extern = (flags &(1 << 1))=0;
        // sym.is_volatile = (flags &1)!=0;

        result.symbols_irep.push(sym);
    }

    // Functions
    let number_of_functions = result.reader.read_cbmc_word();
    debug!("Got {} functions", number_of_functions);
    for _ in 0..number_of_functions {
        let function_name = result.reader.read_gb_string();
        let num_of_instructions = result.reader.read_cbmc_word();

        let mut function = CBMCFunction {
            name: function_name,
            instructions: Vec::with_capacity(num_of_instructions as usize),
        };

        for _ in 0..num_of_instructions {
            // # instructions
            let code = result.reader.read_cbmc_reference();

            let source_location = result.reader.read_cbmc_reference();
            let instr_type = result.reader.read_cbmc_word();
            let guard = result.reader.read_cbmc_reference();

            // Label?
            let target_number = result.reader.read_cbmc_word();

            // Add targets
            let t_count = result.reader.read_cbmc_word();
            let mut targets: Vec<Irept> = Vec::new();
            for _ in 0..t_count {
                // TODO: These should be stored as numbers.
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

        result.functions_irep.push(function);
    }
    result
}

///////////
// TESTS //
///////////

#[cfg(test)]
mod tests {
    #[test]
    fn test_cbmc_to_esbmc_file() {
        let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(err) => panic!("Could not open cargo folder. {}", err),
        };
        let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello-gb.goto");
        assert!(test_path.exists());

        crate::cbmc::process_cbmc_file(test_path.to_str().unwrap());
    }
}
