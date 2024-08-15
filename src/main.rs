use log::{debug, error, info, trace};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::str;

// TODO: Better error handling
// TODO: Cache lookup functions

use env_logger::Env;

#[derive(Clone, Debug)]
struct Irept {
    id: String,
    subt: Vec<Irept>,
    named_subt: HashMap<String, Irept>,
    comments: HashMap<String, Irept>,
}

struct ByteReader {
    file: Vec<u8>,
    pointer: usize,
    irep_container: HashMap<u32, Irept>,
    string_ref_container: HashMap<u32, String>,
}

impl ByteReader {
    fn read_reference(&mut self) -> Irept {
        let id = self.read_u32();
        if self.irep_container.contains_key(&id) {
            return self.irep_container.get(&id).unwrap().clone();
        }

        let irep_id = self.read_string_ref();
        debug!("Got id {}", irep_id);

        let mut irep_sub: Vec<Irept> = Vec::new();
        let mut named_sub: HashMap<String, Irept> = HashMap::new();
        let mut comments_sub: HashMap<String, Irept> = HashMap::new();

        // Sub-expression
        while self.peek() == b'S' {
            self.pointer += 1;
            let sub = self.read_reference();
            irep_sub.push(sub);
        }

        debug!("Got sub {:?}", irep_sub);

        // Named sub
        while self.peek() == b'N' {
            self.pointer += 1;
            let named_id = self.read_string_ref();
            // TODO: assert named_id[0] != '#'
            named_sub.insert(named_id, self.read_reference());
        }

        debug!("Got namedsub {:?}", named_sub);

        // Comment?
        while self.peek() == b'C' {
            self.pointer += 1;
            let named_id = self.read_string_ref();
            // TODO: assert named_id[0] == '#'
            comments_sub.insert(named_id, self.read_reference());
        }

        debug!("Got comments {:?}", comments_sub);

        let end_value = self.get();
        if end_value != 0 {
            panic!("Irep not terminated. Got {}", end_value);
        }

        let result = Irept {
            id: irep_id,
            subt: irep_sub,
            named_subt: named_sub,
            comments: comments_sub,
        };

        self.irep_container.insert(id, result.clone());
        result
    }

    fn peek(&self) -> u8 {
        self.file[self.pointer]
    }

    fn get(&mut self) -> u8 {
        let value = self.file[self.pointer];
        self.pointer += 1;
        value
    }

    fn read_string_ref(&mut self) -> String {
        let id = self.read_u32();

        if self.string_ref_container.contains_key(&id) {
            return self.string_ref_container.get(&id).unwrap().clone();
        }

        let mut bytes = Vec::<u8>::new();
        while self.file[self.pointer] != 0 {
            bytes.push(self.file[self.pointer]);
            self.pointer += 1;
        }
        self.pointer += 1;

        let value = match str::from_utf8(&bytes) {
            Ok(v) => v.to_string(),
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        self.string_ref_container.insert(id, value.clone());
        value
    }

    fn check_header(&mut self) -> bool {
        trace!("Checking header");
        assert!(self.file.len() >= 4);
        let header = vec![self.file[0], self.file[1], self.file[2]];
        let gbf = vec![b'G', b'B', b'F'];
        if header != gbf {
            error!("Invalid header");
            error!("Expected: {}-{}-{}", gbf[0], gbf[1], gbf[2]);
            error!("Found:    {}-{}-{}", header[0], header[1], header[2]);
            return false;
        }
        self.pointer = 3;
        true
    }

    fn check_version(&mut self) -> bool {
        trace!("Checking version");
        self.read_u32() == 1
    }

    fn read_u32(&mut self) -> u32 {
        // Note: a slice might be better here, but then Rust will complain that
        // it can't know the array length statically.
        let raw_bytes = [
            self.file[self.pointer],
            self.file[self.pointer + 1],
            self.file[self.pointer + 2],
            self.file[self.pointer + 3],
        ];
        self.pointer += 4;

        // ESBMC generates this in BE form
        u32::from_be_bytes(raw_bytes)
    }

    // fn read_function(&mut self) -> (String, Irept) {
    //     let name = self.read_string_ref()
    // }
}

fn read_file_as_bytes(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let byte_content = fs::read(path)?;
    Ok(byte_content)
}

fn main() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
    // Input read
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    let byte_content =
        read_file_as_bytes(file_name).expect("Invalid argument: could not read the file");

    let mut reader = ByteReader {
        file: byte_content,
        pointer: 0,
        irep_container: HashMap::new(),
        string_ref_container: HashMap::new(),
    };

    // Check format
    if !reader.check_header() {
        error!("GOTO file in wrong enconding. Exiting");
        return;
    }

    if !reader.check_version() {
        error!("Incompatible GOTO version identified. Exiting");
        return;
    }

    info!("Good to go!");

    // Symbol table
    let number_of_symbols = reader.read_u32();
    let symbols: Vec<Irept> = (0..number_of_symbols)
        .map(|_x| reader.read_reference())
        .collect();
    info!("Got {}. Expected {}", number_of_symbols, symbols.len());

    // Functions
    let _number_of_functions = reader.read_u32();
}
