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

#[derive(Clone, Debug)]
pub struct CBMCFunction {
    pub name: String,
    pub instructions: Vec<Irept>
}


impl From<CBMCSymbol> for Irept {
    fn from(data: CBMCSymbol) -> Self {
        let mut result = Irept::default();
        result.id = String::from("symbol");
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
            .insert("module".to_string(), Irept::from(&data.name));
        result
            .named_subt
            .insert("base_name".to_string(), Irept::from(&data.name));
        result
            .named_subt
            .insert("mode".to_string(), Irept::from(&data.mode));

        // Fix flags
        result
    }
}


#[derive(Clone, Debug)]
pub struct CBMCParser {
    pub reader: ByteReader,
    pub symbols_irep: Vec<Irept>,
    pub functions_irep: Vec<(String, Irept)>,
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

        sym.name = result.reader.read_cbmc_string_ref();
        sym.module = result.reader.read_cbmc_string_ref();
        sym.mode = result.reader.read_cbmc_string_ref();
        sym.base_name = result.reader.read_cbmc_string_ref();
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
        let mut foo_instr = Irept::from("goto-program");
        let foo_name = result.reader.read_gb_string();

        let foo_count = result.reader.read_cbmc_word();
        for _ in 0..foo_count {
            let mut instr = Irept::default();

            instr
                .named_subt
                .insert("code".to_string(), result.reader.read_cbmc_reference());
            instr
                .named_subt
                .insert("location".to_string(), result.reader.read_cbmc_reference());
            instr
                .named_subt
                .insert("typeid".to_string(), result.reader.read_cbmc_reference());
            instr
                .named_subt
                .insert("guard".to_string(), result.reader.read_cbmc_reference());

            let _target_number = result.reader.read_cbmc_word(); // TODO: not sure how to handle this one

            // Add targets
            let t_count = result.reader.read_cbmc_word();
            let mut t_ireps = Irept::default();
            for _ in 0..t_count {
                let target = Irept::from(result.reader.read_cbmc_word().to_string());
                t_ireps.subt.push(target);
            }
            if t_ireps.subt.len() > 0 {
                instr.named_subt.insert("targets".to_string(), t_ireps);
            }

            // Add labels
            let l_count = result.reader.read_cbmc_word();
            let mut l_ireps = Irept::default();
            for _ in 0..l_count {
                let label = result.reader.read_cbmc_string_ref();
                l_ireps.subt.push(Irept::from(label));
            }
            if l_ireps.subt.len() > 0 {
                instr.named_subt.insert("labels".to_string(), l_ireps);
            }

            foo_instr.subt.push(instr);
        }
        let foo = (foo_name, foo_instr);
        result.functions_irep.push(foo.clone());
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

    process_cbmc_file(test_path.to_str().unwrap());
}
