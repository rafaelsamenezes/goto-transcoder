use crate::ByteReader;
use crate::ByteWriter;
pub use crate::Irept;

#[derive(Clone, Debug)]
pub struct ESBMCParseResult {
    pub reader: ByteReader,
    pub symbols_irep: Vec<Irept>,
    pub functions_irep: Vec<(String, Irept)>,
}

pub fn process_esbmc_file(path: &str) -> Result<ESBMCParseResult, String> {
    let mut result = ESBMCParseResult {
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
        let foo = (
            result.reader.read_esbmc_string(),
            result.reader.read_esbmc_reference(),
        );
        result.functions_irep.push(foo.clone());
    }

    return Ok(result);
}

// TODO: ESBMCSymbol and create ESBMCSymbol from CBMCSymbol

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file() {
        let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(err) => panic!("Could not open cargo folder. {}", err),
        };
        let test_path = std::path::Path::new(&cargo_dir).join("resources/test/hello.goto");
        assert!(test_path.exists());

        let result = process_esbmc_file(test_path.to_str().unwrap()).unwrap();

        std::fs::remove_file("/tmp/test.goto").ok();
        ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, "/tmp/test.goto");
    }
}
