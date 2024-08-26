pub use crate::Irept;
use log::{error,trace,debug};
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

impl ByteReader {
    pub fn read_reference(&mut self) -> Irept {
        let id = self.read_u32();
        if self.irep_container.contains_key(&id) {
            return self.irep_container.get(&id).unwrap().clone();
        }

        let irep_id = self.read_string_ref();
        let mut irep_sub: Vec<Irept> = Vec::new();
        let mut named_sub: HashMap<String, Irept> = HashMap::new();
        let mut comments_sub: HashMap<String, Irept> = HashMap::new();

        // Sub-expression
        while self.peek() == b'S' {
            self.pointer += 1;
            let sub = self.read_reference();
            irep_sub.push(sub);
        }

        // Named sub
        while self.peek() == b'N' {
            self.pointer += 1;
            let named_id = self.read_string_ref();
            // TODO: assert named_id[0] != '#'
            named_sub.insert(named_id, self.read_reference());
        }

        // Comment?
        while self.peek() == b'C' {
            self.pointer += 1;
            let named_id = self.read_string_ref();
            // TODO: assert named_id[0] == '#'
            comments_sub.insert(named_id, self.read_reference());
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
        debug!("{}", result);
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

    pub fn read_string(&mut self) -> String {
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

    pub fn read_string_ref(&mut self) -> String {
        let id = self.read_u32();

        if self.string_ref_container.contains_key(&id) {
            return self.string_ref_container.get(&id).unwrap().clone();
        }

        let mut bytes = Vec::<u8>::new();
        while self.peek() != 0 {
            bytes.push(self.get());
        }
        self.pointer += 1;

        let value = String::from_utf8_lossy(&bytes).to_string();

        self.string_ref_container.insert(id, value.clone());
        value
    }

    pub fn check_header(&mut self) -> bool {
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

    pub fn check_version(&mut self) -> bool {
        trace!("Checking version");
        self.read_u32() == 1
    }

    pub fn read_u32(&mut self) -> u32 {
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

    pub fn read_gb_word(&mut self) -> u32{
        let mut shift_distance: u32 = 0;
        let mut res: u32 = 0;
        while self.pointer < self.file.len() {
            if shift_distance >= 32 {
                panic!("input number is too large");
            }

            let byte: u32 = self.get() as u32;
            res = res | ((byte & 0x7f) << shift_distance);
            shift_distance = shift_distance + 7;
            if (byte &  0x80) == 0 {
                break;
            } 
        }

        return res;
    }

    pub fn read_gb_string(&mut self) -> String {
        self.read_string()
    }

    pub fn read_gb_string_ref(&mut self) -> String {
        let id = self.read_gb_word();
        if self.string_ref_container.contains_key(&id) {
            return self.string_ref_container.get(&id).unwrap().clone();
        }
        let value = self.read_gb_string();

        self.string_ref_container.insert(id, value.clone());
        value
    }


    pub fn read_file(path: &str) -> Self {
        trace!("Reading goto file: {}", path);
        let byte_content = fs::read(path).expect("Could not read file");
        ByteReader::from(byte_content)
    }

    pub fn read_gb_reference(&mut self) -> Irept {
        let id = self.read_gb_word();
        if self.irep_container.contains_key(&id) {
            return self.irep_container.get(&id).unwrap().clone();
        }

        let irep_id = self.read_gb_string_ref();
        let mut irep_sub: Vec<Irept> = Vec::new();
        let mut named_sub: HashMap<String, Irept> = HashMap::new();
        let mut comments_sub: HashMap<String, Irept> = HashMap::new();

        // Sub-expression
        while self.peek() == b'S' {
            self.pointer += 1;
            let sub = self.read_gb_reference();
            irep_sub.push(sub);
        }

        // Named sub
        while self.peek() == b'N' {
            self.pointer += 1;
            let named_id = self.read_gb_string_ref();
            // TODO: assert named_id[0] != '#'
            named_sub.insert(named_id, self.read_gb_reference());
        }

        // Comment?
        while self.peek() == b'C' {
            self.pointer += 1;
            let named_id = self.read_gb_string_ref();
            // TODO: assert named_id[0] == '#'
            comments_sub.insert(named_id, self.read_gb_reference());
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
        debug!("{}", result);
        result
    }

    pub fn check_gb_header(&mut self) -> bool {
        trace!("Checking header");
        assert!(self.file.len() >= 4);
        let header = vec![self.file[0], self.file[1], self.file[2],self.file[3]];
        let gbf = vec![0x7f, b'G', b'B', b'F'];
        if header != gbf {
            error!("Invalid header");
            error!("Expected: {}-{}-{}", gbf[0], gbf[1], gbf[2]);
            error!("Found:    {}-{}-{}", header[0], header[1], header[2]);
            return false;
        }
        self.pointer = 4;
        true
    }

    pub fn check_gb_version(&mut self) -> bool {
        trace!("Checking version");        
        self.read_gb_word() == 6
        
    }
    
}

