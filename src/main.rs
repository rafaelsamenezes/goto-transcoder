use esbmc::ESBMCParser;
use log::{debug, error, info, trace};
use std::collections::HashMap;
use std::env::{self, VarError};

// TODO: More functional code when possible
// TODO: Better error handling (learn optional)
// TODO: Cache lookup functions

use env_logger::Env;

pub mod irep;
pub use irep::Irept;

pub mod bytereader;
pub use bytereader::ByteReader;

pub mod esbmc;
pub use esbmc::Symbol;

fn init() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}


fn main() {
    init();

    // Input read
    let args: Vec<String> = env::args().collect();
    let file_name = &args[1];

    
}


