use env_logger::Env;

pub mod irep;
pub use irep::Irept;

pub mod bytereader;
pub use bytereader::ByteReader;

pub mod bytewriter;
pub use bytewriter::ByteWriter;

pub mod sql;

pub mod esbmc;
pub mod cbmc;
fn init() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

fn main() {
    init();    
}


