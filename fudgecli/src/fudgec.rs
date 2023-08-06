mod dotfilegenerator;

use libfudgec::*;

use parser::tokenstream::TokenStream;
use structopt::StructOpt;

use ansi_term::Colour as Color;

#[derive(StructOpt)]
struct CommandLineParameters {
    // Path to file
    #[structopt(parse(from_os_str))]
    main: std::path::PathBuf,

    #[structopt(short = "m", long = "modules", parse(from_os_str))]
    files: Vec<std::path::PathBuf>,

    #[structopt(short = "t", long = "output-tokens")]
    print_tokens: bool,

    #[structopt(short = "a", long = "output-ast")]
    print_ast: bool,
}

fn scan(source: &source::Source, params: &CommandLineParameters) -> scanner::ScannerResult {
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

    scanner_result
}

fn parse(
    source: &source::Source,
    scanner_result: &scanner::ScannerResult,
    ismain: bool,
    params: &CommandLineParameters,
) -> parser::ParserResult {
    let parser_result = parser::parse(
        &mut TokenStream::new(&scanner_result.tokens, &source),
        ismain,
    );

    // Print ast
    if params.print_ast {
        parser_result.ast.print(0);
    }

    parser_result
}

fn scan_and_parse_file<P: AsRef<std::path::Path>>(
    file: P,
    ismain: bool,
    params: &CommandLineParameters,
) -> ast::Ast {
    let source = source::Source::from_file(&file);
    let scanner_result = scan(&source, &params);
    let parser_result = parse(&source, &scanner_result, ismain, &params);

    output::print_errors(&scanner_result.errors, &source);
    output::print_errors(&parser_result.errors, &source);

    parser_result.ast
}

fn main() {
    let params = CommandLineParameters::from_args();

    let mut module_asts: Vec<ast::Ast> = Vec::new();

    println!("{}", Color::Green.bold().paint("Parsing files..."));

    let main_ast = scan_and_parse_file(&params.main, true, &params);

    for path in &params.files {
        module_asts.push(scan_and_parse_file(path, false, &params));
    }

    println!("{}", Color::Green.bold().paint("Generating asg..."));

    let grapher_result = grapher::create_graph(&main_ast, &module_asts);

    // Generate dotfile for asg
    println!("{}", Color::Green.bold().paint("Generating dotfile..."));
    dotfilegenerator::generate_dotfile(
        &grapher_result.asg,
        params.main.file_stem().unwrap().to_str().unwrap(),
    );

    println!("{}", Color::Green.bold().paint("Running treewalker..."));
    interpreter::treewalker::run(&main_ast, &module_asts);

    println!("{}", Color::Green.bold().paint("Generating ircode..."));
    let irprogram = ircodegen::generate_program(&grapher_result.asg);

    println!("{}", Color::Green.bold().paint("Vm Program:"));
    ir::program::print_program(&irprogram);

    println!("{}", Color::Green.bold().paint("Generating vmcode..."));
    let vmprogram = vmcodegen::generate_program(&grapher_result.asg);

    println!("{}", Color::Green.bold().paint("Vm Program:"));
    vm::program::print_program(&vmprogram);

    println!("{}", Color::Green.bold().paint("Running vm..."));
    vm::run(&vmprogram);

    println!("{}", Color::Green.bold().paint("Done"));
}
