use env_logger::Env;

pub mod irep;
pub use irep::Irept;

pub mod bytereader;
pub use bytereader::ByteReader;

pub mod bytewriter;
pub use bytewriter::ByteWriter;
use log::info;

pub mod cbmc;
pub mod esbmc;

fn cbmc2esbmc(input: &str, output: &str) {
    info!("Converting CBMC input into ESBMC");
    let result = crate::cbmc::process_cbmc_file(input);
    std::fs::remove_file(output).ok();
    ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, output);
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
        _ => panic!("Invalid mode: {}", args.mode),
    };

    info!("Done");
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    fn generate_cbmc_gbf(input_c: &str) {
        let goto_cc = match std::env::var("GOTO_CC") {
            Ok(v) => v,
            Err(err) => panic!("Could not get GOTO_CC bin. {}", err),
        };
        assert!(input_c.len() != 0);
        println!("Invoking cbmc with: {}", input_c);

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
        }
        assert_eq!(status, output.status.code().unwrap());
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
        std::fs::remove_file(&esbmc_gbf).ok();
        ByteWriter::write_to_file(result.symbols_irep, result.functions_irep, &esbmc_gbf);

        run_esbmc_gbf(&esbmc_gbf, args, expected);
        std::fs::remove_file("a.out").ok();
        std::fs::remove_file(&esbmc_gbf).ok();
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
        std::fs::remove_file(&esbmc_gbf).ok();
    }

    #[test]
    #[ignore]
    fn hello_world() {
        println!("Remember to set GOTO_CC and ESBMC environment variables!");
        // Basic
        run_test("hello_world.c", &["--goto-functions-only"], 6);
        run_test("hello_world.c", &["--incremental-bmc"], 0);
        run_test("hello_world_fail.c", &["--incremental-bmc"], 1);
        // +
        run_test("hello_add.c", &["--goto-functions-only"], 6);
        run_test("hello_add.c", &["--incremental-bmc"], 0);
        run_test("hello_add_fail.c", &["--incremental-bmc"], 1);
        // -
        run_test("hello_sub.c", &["--goto-functions-only"], 6);
        run_test("hello_sub.c", &["--incremental-bmc"], 0);
        run_test("hello_sub_fail.c", &["--incremental-bmc"], 1);
        // *
        run_test("hello_mul.c", &["--goto-functions-only"], 6);
        run_test("hello_mul.c", &["--incremental-bmc"], 0);
        run_test("hello_mul_fail.c", &["--incremental-bmc"], 1);
        // /
        run_test("hello_div.c", &["--goto-functions-only"], 6);
        run_test("hello_div.c", &["--incremental-bmc"], 0);
        run_test("hello_div_fail.c", &["--incremental-bmc"], 1);
        run_test("hello_div_zero_fail.c", &["--incremental-bmc"], 1);
        run_test(
            "hello_div_zero_fail.c",
            &["--incremental-bmc", "--no-div-by-zero-check"],
            0,
        );
        // ==/!=
        run_test("hello_equality.c", &["--goto-functions-only"], 6);
        run_test("hello_equality.c", &["--incremental-bmc"], 0);
        run_test("hello_equality_fail.c", &["--incremental-bmc"], 1);
        // pointer (address_of)
        run_test("hello_ptr.c", &["--goto-functions-only"], 6);
        run_test("hello_ptr.c", &["--incremental-bmc"], 0);
        run_test("hello_ptr_fail.c", &["--incremental-bmc"], 1);
        // aray
        run_test("hello_array.c", &["--goto-functions-only"], 6);
        run_test("hello_array.c", &["--incremental-bmc"], 0);
        run_test("hello_array_fail.c", &["--goto-functions-only"], 6);
        run_test("hello_array_fail.c", &["--incremental-bmc"], 1);
        run_test("hello_array_fail_oob.c", &["--goto-functions-only"], 6);
        run_test("hello_array_fail_oob.c", &["--incremental-bmc"], 1);
        run_test("hello_array_fail_oob.c", &["--no-bounds-check"], 0);
        // Struct
        run_test("hello_struct.c", &["--goto-functions-only"], 6);
        run_test("hello_struct.c", &["--incremental-bmc"], 0);
        run_test("hello_struct_fail.c", &["--incremental-bmc"], 1);
        // Function call
        run_test("hello_func.c", &["--goto-functions-only"], 6);
        run_test("hello_func.c", &["--incremental-bmc"], 0);
        run_test("hello_func_fail.c", &["--incremental-bmc"], 1);
    }

    #[test]
    #[ignore]
    fn from_rust() {
        // These are example taken from the Kani first steps and then translated into C
        
    }

    #[test]
    #[ignore]
    fn goto_test() {
        run_goto_test("mul.goto", &["--goto-functions-only"], 6);
    }
}
