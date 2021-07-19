#[macro_use]
extern crate tempus_fugit;

use libfudgec::*;

use scanner::Scanner;
use structopt::StructOpt;

#[derive(StructOpt)]
struct CommandLineParameters {
    // Path to file
    #[structopt(parse(from_os_str))]
    file: std::path::PathBuf,

    #[structopt(short = "r", long = "repeats", default_value = "1")]
    repeats: u64,

    #[structopt(short = "t", long = "print-tokens")]
    print_tokens: bool,
}

fn main() {
    let params = CommandLineParameters::from_args();

    let repeats = params.repeats;
    let source = source::FileSource::from_filepath(params.file.clone());

    let mut tokens = Vec::with_capacity(100000);

    let mut total_time = tempus_fugit::Measurement::zero();

    for _i in 0..repeats {
        // Scan all tokens
        tokens.clear();
        let mut scanner = scanner::ScannerImpl::new(&source);
        let (_, measurement) = measure! {{
            while let Some(n) = scanner.read_token() {
                tokens.push(n);
            }
        }};

        total_time = (total_time + measurement).unwrap();

        if params.print_tokens {
            println!("Tokens:");
            print!("    ");
            for t in &tokens {
                print!(
                    "{:?}, ",
                    scanner::token::TokenDisplay {
                        token: t,
                        scanner: &scanner
                    }
                );
            }
            println!("");
        }

        for err in &scanner.errors {
            let lineinfo = scanner.get_line_info(err.source_span.pos).unwrap();
            println!("Error: {} at {}:{}:{}", err.message, params.file.to_str().unwrap(), lineinfo.row, lineinfo.column);

            println!("{} |  {}", lineinfo.row, lineinfo.text.trim());
        }
    }

    println!(
        "Scanned {} tokens in {}, {} times",
        tokens.len(),
        total_time,
        repeats
    );
}
