use std::env;

mod source;
mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
   
    if args.len() < 2 {
        println!("No file supplied");
        return;
    }
    
    let filename = &args[1];
    let source = source::FileSource::new(filename);
    let scanner = scanner::Scanner::new(source);
    scanner.test();
}
