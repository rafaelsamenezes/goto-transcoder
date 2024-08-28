use env_logger::Env;

pub mod irep;
pub use irep::Irept;

pub mod bytereader;
pub use bytereader::ByteReader;

pub mod bytewriter;
pub use bytewriter::ByteWriter;
use log::info;
use sql::SqlReader;
use sql::SqlWriter;

pub mod sql;

pub mod cbmc;
pub mod esbmc;

fn cbmc2esbmc(input: &str, output: &str) {
    info!("Converting CBMC input into ESBMC");
    let result = crate::cbmc::process_cbmc_file(input);
    std::fs::remove_file(output).ok();
    ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, output);
}

fn cbmc2sqlite(input: &str, output: &str) {
    info!("Converting CBMC input into SQLite");
    let result = crate::cbmc::process_cbmc_file(input);
    std::fs::remove_file(output).ok();
    SqlWriter::write_to_file(result.symbols_irep, result.functions_irep, output);
}

fn sqlite2esbmc(input: &str, output: &str) {
    info!("Converting SQLite input into ESBMC");
    let result = SqlReader::open(input);
    std::fs::remove_file(output).ok();
    ByteWriter::write_to_file(result.get_symbols(), result.get_functions(), output);
}

fn esbmc2sqlite(input: &str, output: &str) {
    info!("Converting ESBMC input into SQLite");
    let result = crate::esbmc::process_esbmc_file(input).unwrap();
    std::fs::remove_file(output).ok();
    SqlWriter::write_to_file(result.symbols_irep, result.functions_irep, output);
}

fn init() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Qwe
    #[arg(short, long)]
    mode: u8, // TODO: this should be an enum
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
}

fn main() {
    init();
    let args = Args::parse();

    match args.mode {
        0 => cbmc2esbmc(&args.input, &args.output),
        1 => cbmc2sqlite(&args.input, &args.output),
        2 => sqlite2esbmc(&args.input, &args.output),
        3 => esbmc2sqlite(&args.input, &args.output),
        _ => panic!("Invalid mode: {}", args.mode),
    };

    info!("Done");
}
