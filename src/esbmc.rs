use crate::ByteReader;
pub use crate::Irept;

#[derive(Clone, Debug)]
pub struct Symbol {
    t: Irept,
    value: Irept,
    location: Irept,
    id: String,
    module: String,
    name: String,
    mode: String,

    is_type: bool,
    is_macro: bool,
    is_parameter: bool,

    lvalue: bool,
    static_lifetime: bool,
    file_local: bool,
    is_extern: bool,
}

impl From<Irept> for Symbol {
    fn from(i: Irept) -> Self {
        Symbol {
            t: i.get_type().clone(),
            value: i.get_symvalue().clone(),
            location: i.get_location(),
            id: i.get_name(),
            module: i.get_module(),
            name: i.get_base_name(),
            mode: i.get_mode(),
            is_type: i.is_type(),
            is_macro: i.is_macro(),
            is_parameter: i.is_parameter(),
            lvalue: i.is_lvalue(),
            static_lifetime: i.is_static_lifetime(),
            file_local: i.is_file_local(),
            is_extern: i.is_extern(),
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Symbol: {}", self.id)
    }
}



#[derive(Clone, Debug)]
pub struct Function {
    name: String,
    body: Option<Irept>
}


impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Function: {}", self.name)
    }
}

#[derive(Clone, Debug)]
pub struct ESBMCParser {
    pub reader: ByteReader,
    pub symbols: Vec<Symbol>,
    pub functions: Vec<Function>
}

#[derive(Clone, Debug)]
pub enum ESBMCParserError {
    InvalidEncoding,
    InvalidVersion { version: u32 },
    CouldNotParseSymbols { expected: u32, actual: u32 },
}

impl std::fmt::Display for ESBMCParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ESBMCParserError::InvalidEncoding => write!(f, "Unsupported encoding, expected 'GBF'"),
            ESBMCParserError::InvalidVersion { version } => {
                write!(f, "Expected version 1, found {}", version)
            }
            ESBMCParserError::CouldNotParseSymbols { expected, actual } => write!(
                f,
                "Could not parse all symbols. Expected {}, found {}",
                expected, actual
            ),
        }
    }
}

pub fn process_file(path: &str) -> Result<ESBMCParser, ESBMCParserError> {
    let mut result = ESBMCParser {
        reader: ByteReader::read_file(path),
        symbols: Vec::new(),
        functions: Vec::new(),
    };

    if !result.reader.check_header() {
        return Err(ESBMCParserError::InvalidEncoding);
    }

    if !result.reader.check_version() {
        return Err(ESBMCParserError::InvalidVersion { version: 2 });
    }

    // Symbol table
    let number_of_symbols = result.reader.read_u32();
    for _ in 0..number_of_symbols {
        result.symbols.push(Symbol::from(result.reader.read_reference()));
    }

    // Functions
    let number_of_functions = result.reader.read_u32();
    for _ in 0..number_of_functions {
        let foo = Function{ name: result.reader.read_string(), body: Some(result.reader.read_reference())};
        result.functions.push(foo);
    }

    return Ok(result);
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
    let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello.goto");
    assert!(test_path.exists());

    let result = process_file(test_path.to_str().unwrap()).unwrap();
     for s in result.symbols {
        log::debug!("{}", s);
    }

     for f in result.functions {
        log::debug!("{}", f);
    }
}
