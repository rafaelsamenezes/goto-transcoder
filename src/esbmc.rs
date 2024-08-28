use std::collections::HashMap;

use log::debug;

use crate::cbmc::CBMCSymbol;
pub use crate::Irept;
use crate::{ByteReader, ByteWriter};

#[derive(Clone, Debug)]
pub struct ESBMCParser {
    pub reader: ByteReader,
    pub symbols_irep: Vec<Irept>,
    pub functions_irep: Vec<(String, Irept)>,
}

pub fn process_file(path: &str) -> Result<ESBMCParser, String> {
    let mut result = ESBMCParser {
        reader: ByteReader::read_file(path),
        functions_irep: Vec::new(),
        symbols_irep: Vec::new(),
    };

    result.reader.check_esbmc_header().unwrap();
result.reader.check_esbmc_version().unwrap();

    // Symbol table
    let number_of_symbols = result.reader.read_esbmc_word();
    for _ in 0..number_of_symbols {
        let symbol = result.reader.read_esbmc_reference();
        result.symbols_irep.push(symbol.clone());
    }

    // Functions
    let number_of_functions = result.reader.read_esbmc_word();
    for _ in 0..number_of_functions {
        let foo = (result.reader.read_esbmc_string(), result.reader.read_esbmc_reference());
        result.functions_irep.push(foo.clone());
    }

    return Ok(result);
}

// TODO: ESBMCSymbol and create ESBMCSymbol from CBMCSymbol

#[test]
fn test_file() {
    let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(err) => panic!("Could not open cargo folder. {}", err),
    };
    let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello.goto");
    assert!(test_path.exists());

    let result = process_file(test_path.to_str().unwrap()).unwrap();

    std::fs::remove_file("test.goto").ok();
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
    std::fs::remove_file("test.sqlite3").ok();
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

    std::fs::remove_file("sqlite3_test.goto").ok();
    ByteWriter::write_to_file(symbols, functions, "sqlite3_test.goto");
}


