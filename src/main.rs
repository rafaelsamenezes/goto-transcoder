mod adapter;
mod bytereader;
mod bytewriter;
mod cbmc;
mod esbmc;
mod irep;
mod resources;

pub use adapter::cbmc2esbmc;
pub use bytereader::ByteReader;
pub use bytewriter::ByteWriter;
use esbmc::ESBMCParseResult;
pub use irep::Irept;

use log::trace;

use clap::{Args, Parser, Subcommand};

fn init() {
    use env_logger::Env;
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Converts CBMC <INPUT> into ESBMC <OUTPUT>
    CBMC2ESBMC(CmdArgs),
    /// Converts CBMC <INPUT> into JSON <OUTPUT>
    CBMC2JSON(CmdArgs),
    /// Converts ESBMC <INPUT> into CBMC <OUTPUT>
    ESBMC2CBMC(CmdArgs),
    /// Converts ESBMC <INPUT> into JSON <OUTPUT>
    ESBMC2JSON(CmdArgs),
    /// Converts JSON <INPUT> into CBMC <OUTPUT>
    JSON2CBMC(CmdArgs),
    /// Converts JSON <INPUT> into ESBMC <OUTPUT>
    JSON2ESBMC(CmdArgs),
}

#[derive(Args)]
struct CmdArgs {
    entrypoint: String,
    input: std::path::PathBuf,
    output: std::path::PathBuf,
}

fn main() {
    init();
    trace!("Starting goto-transcoder");
    let cli = Cli::parse();

    match cli.command {
        Commands::CBMC2ESBMC(args) => {
            cbmc2esbmc(
                &args.entrypoint,
                &args.input.to_str().unwrap(),
                args.output.to_str().unwrap(),
            );
        }
        _ => panic!("Command not implemented yet"),
    };

    trace!("Done");
}
