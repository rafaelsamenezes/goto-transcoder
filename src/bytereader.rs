pub use crate::Irept;
use log::trace;
use std::collections::HashMap;
use std::fs;
use std::str;

#[derive(Clone, Debug)]
pub struct ByteReader {
    file: Vec<u8>,
    pointer: usize,
    irep_container: HashMap<u32, Irept>,
    string_ref_container: HashMap<u32, String>,
}

impl From<Vec<u8>> for ByteReader {
    fn from(data: Vec<u8>) -> Self {
        ByteReader {
            file: data,
            pointer: 0,
            irep_container: HashMap::new(),
            string_ref_container: HashMap::new(),
        }
    }
}

// NOTE: There is a lot of code duplication. I don't really care at the moment as we just have
//       two GBF formats.

impl ByteReader {
    // File manipulation

    pub fn read_file(path: &str) -> Self {
        trace!("Reading goto file: {}", path);
        let byte_content = fs::read(path).expect("Could not read file");
        ByteReader::from(byte_content)
    }

    fn peek(&self) -> u8 {
        self.file[self.pointer]
    }

    fn get(&mut self) -> u8 {
        let value = self.file[self.pointer];
        self.pointer += 1;
        value
    }

    // Reference parsing. First try the cache, if not available then parse the irep

    pub fn read_esbmc_reference(&mut self) -> Irept {
        let id = self.read_esbmc_word();
        if self.irep_container.contains_key(&id) {
            return self.irep_container.get(&id).unwrap().clone();
        }

        let irep_id = self.read_esbmc_string_ref();

        // Sub-expression
        let mut irep_sub: Vec<Irept> = Vec::new();
        while self.peek() == b'S' {
            self.pointer += 1;
            let sub = self.read_esbmc_reference();
            irep_sub.push(sub);
        }

        // Named sub
        let mut named_sub: HashMap<String, Irept> = HashMap::new();
        while self.peek() == b'N' {
            self.pointer += 1;
            let named_id = self.read_esbmc_string_ref();
            // TODO: assert named_id[0] != '#'
            named_sub.insert(named_id, self.read_esbmc_reference());
        }

        // Comment?
        let mut comments_sub: HashMap<String, Irept> = HashMap::new();
        while self.peek() == b'C' {
            self.pointer += 1;
            let named_id = self.read_esbmc_string_ref();
            // TODO: assert named_id[0] == '#'
            comments_sub.insert(named_id, self.read_esbmc_reference());
        }

        let end_value = self.get();
        if end_value != 0 {
            panic!("Irep not terminated.");
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

    pub fn read_cbmc_reference(&mut self) -> Irept {
        let id = self.read_cbmc_word();
        if self.irep_container.contains_key(&id) {
            return self.irep_container.get(&id).unwrap().clone();
        }

        let irep_id = self.read_cbmc_string_ref();

        // Sub-expression
        let mut irep_sub: Vec<Irept> = Vec::new();
        while self.peek() == b'S' {
            self.pointer += 1;
            let sub = self.read_cbmc_reference();
            irep_sub.push(sub);
        }

        // Named sub
        let mut named_sub: HashMap<String, Irept> = HashMap::new();
        while self.peek() == b'N' {
            self.pointer += 1;
            let named_id = self.read_cbmc_string_ref();
            // TODO: assert named_id[0] != '#'
            named_sub.insert(named_id, self.read_cbmc_reference());
        }

        // Comment?
        let mut comments_sub: HashMap<String, Irept> = HashMap::new();
        while self.peek() == b'C' {
            self.pointer += 1;
            let named_id = self.read_cbmc_string_ref();
            // TODO: assert named_id[0] == '#'
            comments_sub.insert(named_id, self.read_cbmc_reference());
        }

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

    // String parsing.

    pub fn read_esbmc_string(&mut self) -> String {
        let mut bytes = Vec::<u8>::new();
        while self.peek() != 0 {
            let c = self.get();
            if c == b'\\' {
                bytes.push(self.get());
            } else {
                bytes.push(c);
            }
        }
        self.pointer += 1;
        let value = String::from_utf8_lossy(&bytes).to_string();
        value
    }

    pub fn read_gb_string(&mut self) -> String {
        self.read_esbmc_string()
    }

    // String reference parsing. Similar than the irep one

    pub fn read_esbmc_string_ref(&mut self) -> String {
        let id = self.read_esbmc_word();

        if self.string_ref_container.contains_key(&id) {
            return self.string_ref_container.get(&id).unwrap().clone();
        }

        let value = self.read_esbmc_string();

        self.string_ref_container.insert(id, value.clone());
        value
    }

    pub fn read_cbmc_string_ref(&mut self) -> String {
        let id = self.read_cbmc_word();
        if self.string_ref_container.contains_key(&id) {
            return self.string_ref_container.get(&id).unwrap().clone();
        }
        let value = self.read_gb_string();

        self.string_ref_container.insert(id, value.clone());
        value
    }

    // Word reading (as u32)

    pub fn read_esbmc_word(&mut self) -> u32 {
        // TODO: a slice might be better here, but then Rust will complain that
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

    pub fn read_cbmc_word(&mut self) -> u32 {
        let mut shift_distance: u32 = 0;
        let mut res: u32 = 0;
        while self.pointer < self.file.len() {
            if shift_distance >= 32 {
                panic!("input number is too large");
            }

            let byte: u32 = self.get() as u32;
            res = res | ((byte & 0x7f) << shift_distance);
            shift_distance = shift_distance + 7;
            if (byte & 0x80) == 0 {
                break;
            }
        }

        if self.pointer > self.file.len() {
            panic!("Unexpected end of stream");
        }

        return res;
    }

    // GBF checks

    pub fn check_esbmc_header(&mut self) -> Result<(), String> {
        trace!("Checking header");
        assert!(self.file.len() >= 4);
        let header = vec![self.file[0], self.file[1], self.file[2]];
        let gbf = vec![b'G', b'B', b'F'];
        if header != gbf {
            return Err(format!(
                "Invalid ESBMC header. Found: {}{}{}",
                header[0], header[1], header[2]
            ));
        }
        self.pointer = 3;
        Ok(())
    }

    pub fn check_cbmc_header(&mut self) -> Result<(), String> {
        trace!("Checking header");
        assert!(self.file.len() >= 4);
        let header = vec![self.file[0], self.file[1], self.file[2], self.file[3]];
        let gbf = vec![0x7f, b'G', b'B', b'F'];
        if header != gbf {
            return Err(format!(
                "Invalid CBMC header. Found: {}{}{}{}",
                header[0], header[1], header[2], header[3]
            ));
        }
        self.pointer = 4;
        Ok(())
    }

    pub fn check_esbmc_version(&mut self) -> Result<(), String> {
        let version = self.read_esbmc_word();
        if version != 1 {
            return Err(format!("Invalid ESBMC version. Found {}", version));
        }
        Ok(())
    }

    pub fn check_cbmc_version(&mut self) -> Result<(), String> {
        let version = self.read_cbmc_word();
        if version != 6 {
            return Err(format!("Invalid CBMC version. Found {}", version));
        }
        Ok(())
    }
}
