use std::env;
use std::fs;

// TODO: Read symbols
// TODO: Better error handling

struct ByteReader {
    file: Vec<u8>,
    pointer: usize,
}

impl ByteReader {
    fn check_header(&mut self) -> bool {
        // TODO: This should go for take
        assert!(self.file.len() >= 4);
        let header = vec![self.file[0], self.file[1], self.file[2]];
        let gbf = vec![b'G', b'B', b'F'];
        if header != gbf {
            println!("{}-{}-{}", gbf[0], gbf[1], gbf[2]);
            println!("{}-{}-{}", header[0], header[1], header[2]);
            return false;
        }
        self.pointer = 3;
        true
    }

    fn check_version(&mut self) -> bool {
        self.read_u32() == 1
    }

    fn read_u32(&mut self) -> u32 {
        let raw_bytes = [
            self.file[self.pointer],
            self.file[self.pointer + 1],
            self.file[self.pointer + 2],
            self.file[self.pointer + 3],
        ];

        // ESBMC generates this in BE form
        let num = u32::from_be_bytes(raw_bytes);
        self.pointer += 4;
        num
    }
}

fn read_file_as_bytes(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let byte_content = fs::read(path)?;
    Ok(byte_content)
}

fn main() {
    // Input read
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    let byte_content =
        read_file_as_bytes(file_name).expect("Invalid argument: could not read the file");

    let mut reader = ByteReader {
        file: byte_content,
        pointer: 0,
    };

    // Check format

    if !reader.check_header() {
        println!("GOTO file in wrong enconding. Exiting");
        return;
    }

    if !reader.check_version() {
        println!("Incompatible GOTO version identified. Exiting");
    }

    // Symbol table
    let _number_of_symbols = reader.read_u32();

    // Functions
    let _number_of_functions = reader.read_u32();
}
