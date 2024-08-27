use crate::{ByteWriter, ByteReader};
pub use crate::Irept;
use log::{error,trace,debug};


#[derive(Clone, Debug)]
pub struct CBMCSymbol {
    pub stype: Irept,
    pub value: Irept,
    pub location: Irept,
    pub name: String,
    pub module: String,
    pub base_name: String,
    pub mode:  String,
    pub pretty_name: String,
    pub flags: u32    
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
            flags: 0
        }
    }
}



#[derive(Clone, Debug)]
pub struct CBMCParser {
    pub reader: ByteReader,
    pub symbols_irep: Vec<CBMCSymbol>,
    pub functions_irep: Vec<(String, Vec<Irept>)>,
}

pub fn process_gb_file(path: &str) -> CBMCParser {
    let mut result = CBMCParser {
        reader: ByteReader::read_file(path),
        functions_irep: Vec::new(),
        symbols_irep: Vec::new(),
    };

    if !result.reader.check_gb_header() {
        panic!("Invalid header");
    }

    if !result.reader.check_gb_version() {
        panic!("Invalid version");
    }

    // Symbol table
    let number_of_symbols = result.reader.read_gb_word();
    debug!("Got {} symbols", number_of_symbols);
    
    for _ in 0..number_of_symbols {
        let mut sym = CBMCSymbol::default();
        sym.stype = result.reader.read_gb_reference();
        sym.value = result.reader.read_gb_reference();
        sym.location = result.reader.read_gb_reference();

        sym.name = result.reader.read_gb_string_ref();
        sym.module = result.reader.read_gb_string_ref();
        sym.mode = result.reader.read_gb_string_ref();
        sym.base_name = result.reader.read_gb_string_ref();
        sym.pretty_name = result.reader.read_gb_string_ref();

        result.reader.read_gb_word();
        sym.flags = result.reader.read_gb_word();
        debug!("Symbol name {}", sym.name);
        result.symbols_irep.push(sym);
    }
    

    // Functions
    let number_of_functions = result.reader.read_gb_word();
    debug!("Got {} functions", number_of_functions);
    for _ in 0..number_of_functions {
        let foo_name = result.reader.read_gb_string();        
        let foo_count = result.reader.read_gb_word();
        debug!("Got {} function with {} instr", foo_name, foo_count);
        
        for _ in 0..foo_count {
            let _code = result.reader.read_gb_reference();
            let _source_location = result.reader.read_gb_reference();
            let _instruction_type = result.reader.read_gb_word();
            let _guard = result.reader.read_gb_reference();
            let _target_number = result.reader.read_gb_word();

            let _t_count = result.reader.read_gb_word();
            for _ in 0.._t_count {
                let _target = result.reader.read_gb_word();
            }
            let _l_count = result.reader.read_gb_word();
            for _ in 0.._l_count {
                let _label = result.reader.read_gb_string_ref();
            }
        }
        //let foo = (result.reader.read_string(), result.reader.read_reference());
        //result.functions_irep.push(foo.clone());
    }

    result
}

#[test]
fn test_file() {
    let env = env_logger::Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello-gb.goto");
    assert!(test_path.exists());

    process_gb_file(test_path.to_str().unwrap());
}
