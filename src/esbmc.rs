use crate::{ByteWriter, ByteReader};
pub use crate::Irept;


#[derive(Clone, Debug)]
pub struct ESBMCParser {
    pub reader: ByteReader,
    pub symbols_irep: Vec<Irept>,
    pub functions_irep: Vec<(String, Irept)>,
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
        functions_irep: Vec::new(),
        symbols_irep: Vec::new(),
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
        let symbol = result.reader.read_reference();
        result.symbols_irep.push(symbol.clone());
    }

    // Functions
    let number_of_functions = result.reader.read_u32();
    for _ in 0..number_of_functions {
        let foo = (result.reader.read_string(), result.reader.read_reference());
        result.functions_irep.push(foo.clone());
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
    ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, "test.goto");
}

use crate::sql::SqlWriter;
#[test]
fn test_write_sql_file() {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    
    let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello.goto");
    assert!(test_path.exists());

    let result = process_file(test_path.to_str().unwrap()).unwrap();    
    SqlWriter::write_to_file(result.symbols_irep, result.functions_irep, "test.sqlite3");
}

use crate::sql::SqlReader;
#[test]
fn test_read_sql_file() {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
     };
    
    let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello.sqlite3");
    assert!(test_path.exists());

    let reader = SqlReader::open(test_path.to_str().unwrap());

    let symbols = reader.get_symbols();
    let functions = reader.get_functions();
    
    ByteWriter::write_to_file(symbols, functions, "sqlite3_test.goto");
}
