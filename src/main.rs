use std::env;

mod source;
mod scanner;
mod token;

use source::Source;

fn main() {
    let args: Vec<String> = env::args().collect();
   
    if args.len() < 2 {
        println!("No file supplied");
        return;
    }

    let filename = &args[1];
    let source = source::FileSource::new(filename);

    println!("Characters in file:");
    print!("    ");
    let mut reader = source.get_reader();
    while let Some(n) = reader.peek() {
        print!("{}, ", n as char);
        reader.advance();
    }
    println!("");

    println!("Tokens:");
    print!("    ");
    let mut scanner = scanner::Scanner::new(&source);
    while let Some(n) = scanner.read_token() {
        print!("{:?}, ", n);
    }

    println!("\nDone");
}
