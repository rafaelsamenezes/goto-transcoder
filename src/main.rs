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

#[cfg(test)]
mod tests {
    use std::{os::unix::process::ExitStatusExt, process::Command};

    fn generate_cbmc_gbf(input_c: &str) {
        let goto_cc = match std::env::var("GOTO_CC") {
            Ok(v) => v,
            Err(err) => panic!("Could not get GOTO_CC bin. {}", err),
        };
        assert!(input_c.len() != 0);

        let output = Command::new(goto_cc)
            .arg(input_c)
            .output()
            .expect("failed to execute process");

        if !output.status.success() {
            println!("CBMC exited with {}", output.status);
            println!(
                "\tSTDOUT: {}",
                String::from_utf8_lossy(&output.stdout).to_string()
            );
            println!(
                "\tSTDERR: {}",
                String::from_utf8_lossy(&output.stderr).to_string()
            );
            panic!("GOTO-CC failed");
        }
    }

    fn run_esbmc_gbf(input_gbf: &str, args: &[&str], status: i32) {
        let esbmc = match std::env::var("ESBMC") {
            Ok(v) => v,
            Err(err) => panic!("Could not get ESBMC bin. {}", err),
        };
        let output = Command::new(esbmc)
            .arg("--binary")
            .arg(input_gbf)
            .args(args)
            .output()
            .expect("Failed to execute process");

        if !output.status.success() {
            
            println!("ESBMC exited with {}", output.status);
            println!(
                "\tSTDOUT: {}",
                String::from_utf8_lossy(&output.stdout).to_string()
            );
            println!(
                "\tSTDERR: {}",
                String::from_utf8_lossy(&output.stderr).to_string()
            );
            assert_eq!(status, output.status.code().unwrap());
        }
    }

    use crate::cbmc;
    use crate::ByteWriter;

    fn run_test(input_c: &str, args: &[&str], expected: i32) {
        let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(err) => panic!("Could not open cargo folder. {}", err),
        };
        let test_path =
            std::path::Path::new(&cargo_dir).join(format!("resources/test/{}", input_c));
        let esbmc_gbf = format!("{}.goto", input_c); // TODO: generate UUID!

        generate_cbmc_gbf(test_path.to_str().unwrap());

        let result = cbmc::process_cbmc_file("a.out");
        std::fs::remove_file("a.out").ok();
        std::fs::remove_file(&esbmc_gbf).ok();
        ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, &esbmc_gbf);

        run_esbmc_gbf(&esbmc_gbf, args, expected);
        //std::fs::remove_file(&esbmc_gbf).ok();
    }

    fn run_goto_test(input_goto: &str, args: &[&str], expected: i32) {
        let cargo_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(v) => v,
            Err(err) => panic!("Could not open cargo folder. {}", err),
        };
        let test_path =
            std::path::Path::new(&cargo_dir).join(format!("resources/test/{}", input_goto));
        let result = cbmc::process_cbmc_file(test_path.to_str().unwrap());

        let esbmc_gbf = format!("{}.goto", input_goto); // TODO: generate UUID!
        std::fs::remove_file(&esbmc_gbf).ok();
        ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, &esbmc_gbf);
        run_esbmc_gbf(&esbmc_gbf, args, expected);
    }

    #[test]
    #[ignore]
    fn hello_world() {
         run_test("hello_world.c", &["--goto-functions-only"], 6);
         run_test("hello_world.c", &["--incremental-bmc"], 0);
         run_test("hello_world_fail.c", &["--incremental-bmc"], 1);
         run_test("hello_struct.c", &["--incremental-bmc"], 1);
         run_test("hello_anon_struct.c", &["--incremental-bmc"], 1);
    }

    #[test]
    #[ignore]
    fn mk_mul() {
        run_goto_test("mul.goto", &["--goto-functions-only"], 6);
        run_goto_test("mul_contract.goto", &["--goto-functions-only"], 6);
    }

    #[test]
    #[ignore]
    fn basic_exprs() {
        // TODO:  "member", "typecast", "notequal", "or", "mod", "not", "*", "/", "+", "-", "=", "<", "lshr", "shl", "address_of", "index", "pointer_object", "array_of", "sideeffect", "dereference", "bitand"

    }

    
}
