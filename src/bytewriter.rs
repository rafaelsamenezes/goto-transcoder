pub use crate::Irept;
use log::trace;
use log::debug;
use std::collections::HashMap;
use std::io::Write;

pub struct ByteWriter {
    file: Vec<u8>,
    irep_container: HashMap<Irept, u32>,
    string_ref_container: HashMap<String, u32>,
}

impl ByteWriter {
    pub fn write_to_file(symbols: Vec<Irept>, functions: Vec<(String, Irept)>, output: &str) {
        trace!("Writing goto file: {}", output);
        let mut writer = ByteWriter {
            file: Vec::new(),
            irep_container: HashMap::new(),
            string_ref_container: HashMap::new(),
        };
        writer.file.push(b'G');
        writer.file.push(b'B');
        writer.file.push(b'F');
        writer.write_u32(1);

        // Add symbols
        writer.write_u32(symbols.len() as u32);
        for irep in symbols {
            debug!("writing symbol {}", &irep);
            writer.write_reference(&irep);
        }

        // Add functions
        writer.write_u32(functions.len() as u32);
        for (name, irep) in functions {
            debug!("writing function {}: {}", &name, &irep);
            writer.write_string(&name);
            writer.write_reference(&irep);
        }

        let mut file = std::fs::File::create(output).unwrap();
        file.write_all(&writer.file).unwrap();
    }

    fn write_string(&mut self, value: &str) {
        // TODO: Can we add a range into a vector directly?
        for byte in value.as_bytes() {
            self.file.push(byte.clone());
        }
        self.file.push(0);
    }

    fn write_u32(&mut self, value: u32) {
        // TODO: Maybe there is a better way
        value.to_be_bytes().map(|b| self.file.push(b));
    }

    fn write_irep(&mut self, value: &Irept) {
        self.write_string_reference(&value.id);
        for irep in &value.subt {
            self.file.push(b'S');
            self.write_reference(irep);
        }

        for (name, irep) in &value.named_subt {
            self.file.push(b'N');
            self.write_string_reference(name);
            self.write_reference(irep);
        }

        for (name, irep) in &value.comments {
            self.file.push(b'C');
            self.write_string_reference(name);
            self.write_reference(irep);
        }

        self.file.push(0);
    }

    fn write_reference(&mut self, value: &Irept) {
        if self.irep_container.contains_key(value) {
            let id = self.irep_container[value];
            self.write_u32(id);
            return;
        }
        let id = self.irep_container.len() as u32;
        self.irep_container.insert(value.clone(), id);
        self.write_u32(id);
        self.write_irep(value);
    }
    fn write_string_reference(&mut self, value: &str) {
        if self.string_ref_container.contains_key(value) {
            let id = self.string_ref_container[value];
            self.write_u32(id);
            return;
        }
        let id = self.string_ref_container.len() as u32;
        self.string_ref_container.insert(String::from(value), id);
        self.write_u32(id);
        self.write_string(value);
    }
}
