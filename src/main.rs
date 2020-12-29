use structopt::StructOpt;

mod source;
mod scanner;
mod token;

use source::Source;

#[derive(StructOpt)]
struct CommandLineParameters {
    // Path to file
    #[structopt(parse(from_os_str))]
    file: std::path::PathBuf,

    #[structopt(short = "s", long = "print-source")]
    print_source: bool,

    #[structopt(short = "t", long = "print-tokens")]
    print_tokens: bool,
}

fn main() {
    let params = CommandLineParameters::from_args();

    let source = source::FileSource::new(params.file);

    if params.print_source {
        println!("Characters in file:");
        print!("    ");
        let mut reader = source.get_reader();
        while let Some(n) = reader.peek() {
            print!("{}, ", n as char);
            reader.advance();
        }
        println!("");
    }

    if params.print_tokens {
        println!("Tokens:");
        print!("    ");
        let mut scanner = scanner::Scanner::new(&source);
        while let Some(n) = scanner.read_token() {
            print!("{:?}, ", n);
        }
    }

    println!("\nDone");
}
