use libfudgec::*;

use crate::parser::tokenstream::TokenStream;
use structopt::StructOpt;

use ansi_term::Colour as Color;

#[derive(StructOpt)]
struct CommandLineParameters {
    // Path to file
    #[structopt(parse(from_os_str))]
    file: std::path::PathBuf,

    #[structopt(short = "t", long = "output-tokens")]
    print_tokens: bool,
}

fn main() {
    let params = CommandLineParameters::from_args();

    let source = source::Source::from_file(params.file);

    // If printing tokens, do a separate scan for the print
    if params.print_tokens {
        let scanner_result = scanner::tokenize(&source);

        println!("Tokens:");
        print!("    ");
        for t in scanner_result.tokens {
            print!(
                "{:?}, ",
                scanner::token::TokenDisplay {
                    token: &t,
                    source: &source,
                }
            );
        }

        println!("");
    }

    // Scan and parse
    let scanner_result = scanner::tokenize(&source);
    let parser_result = parser::parse(&mut TokenStream::new(&scanner_result.tokens, &source));

    let mut treewalker = interpreter::TreeWalker::new(&parser_result.ast);
    treewalker.interpret();

    // Print errors
    output::print_errors(&scanner_result.errors, &source);
    output::print_errors(&parser_result.errors, &source);

    println!("{}", Color::Green.bold().paint("Done"));
}
