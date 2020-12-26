mod source;
mod scanner;

fn main() {
    println!("Hello, world, för fan!");
    let source = source::ByteArraySource::new("Hejhopp".as_bytes());
    let scanner = scanner::Scanner::new(source);
    scanner.test();
}
