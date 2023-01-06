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

    #[structopt(short = "a", long = "output-ast")]
    print_ast: bool,
}

fn main() {
    let params = CommandLineParameters::from_args();

    let source = source::Source::from_file(params.file);

    // Scan and parse
    let scanner_result = scanner::tokenize(&source);

    // Print tokens
    if params.print_tokens {
        println!("Tokens:");
        print!("    ");
        for t in &scanner_result.tokens {
            print!(
                "{:?}, ",
                scanner::token::TokenDisplay {
                    token: t,
                    source: &source,
                }
            );
        }

        println!("");
    }

    let parser_result = parser::parse(&mut TokenStream::new(&scanner_result.tokens, &source));

    // Print ast
    if params.print_ast {
        parser_result.ast.print(0);
    }

    // Print errors
    output::print_errors(&scanner_result.errors, &source);
    output::print_errors(&parser_result.errors, &source);

    let mut treewalker = interpreter::TreeWalker::new(&parser_result.ast);
    treewalker.interpret();

    println!("{}", Color::Green.bold().paint("Done"));
}
