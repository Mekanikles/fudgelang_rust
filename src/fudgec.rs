#[macro_use]
extern crate tempus_fugit;

use libfudgec::*;

use scanner::Scanner;
use structopt::StructOpt;

use ansi_term::Colour as Color;

#[derive(StructOpt)]
struct CommandLineParameters {
    // Path to file
    #[structopt(parse(from_os_str))]
    file: std::path::PathBuf,

    #[structopt(short = "r", long = "bench-repeats", default_value = "0")]
    bench_repeats: u64,

    #[structopt(short = "t", long = "output-tokens")]
    print_tokens: bool,
}

fn main() {
    let params = CommandLineParameters::from_args();

    // Bench mode
    if params.bench_repeats > 0 {
        let repeats = params.bench_repeats;
        let mut tokens = Vec::with_capacity(100000);

        let mut total_scan_time = tempus_fugit::Measurement::zero();

        let source = source::FileSource::from_filepath(params.file.clone());

        for _i in 0..repeats {
            // Scan all tokens
            tokens.clear();
            let mut scanner = scanner::ScannerImpl::new(&source);
            let (_, measurement) = measure! {{
                while let Some(t) = scanner.read_token() {
                    tokens.push(t);
                }
            }};

            total_scan_time = (total_scan_time + measurement).unwrap();
        }

        println!(
            "Scanned {} tokens in {}, {} times. ({} per scan)",
            tokens.len(),
            total_scan_time,
            repeats,
            tempus_fugit::Measurement::from(
                tempus_fugit::Duration::from(total_scan_time.clone()) / repeats as i32
            )
        );
    }

    let source = source::FileSource::from_filepath(params.file.clone());

    // If printing tokens, do a separate scan for the print
    if params.print_tokens {
        let mut scanner = scanner::ScannerImpl::new(&source);

        println!("Tokens:");
        print!("    ");
        while let Some(t) = scanner.read_token() {
            print!(
                "{:?}, ",
                scanner::token::TokenDisplay {
                    token: &t,
                    scanner: &scanner
                }
            );
        }
        println!("");
    }

    // Scan and parse
    let mut scanner = scanner::ScannerImpl::new(&source);
    let mut parser = parser::Parser::new(&mut scanner);
    parser.parse();
    parser.print_ast();

    let mut treewalker = interpreter::TreeWalker::new(&parser.ast);
    treewalker.interpret();

    // Print errors
    // TODO: Have to print in this order, since parse borrows scanner
    output::print_errors(parser.get_errors(), &source);
    output::print_errors(scanner.get_errors(), &source);

    println!("{}", Color::Green.bold().paint("Done"));
}
