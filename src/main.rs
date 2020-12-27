mod source;
mod scanner;

fn main() {
    println!("Hello, world, för fan!");
    let source = source::ByteArraySource::new("Hejhopp".as_bytes());
    let scanner = scanner::Scanner::new(source);
    scanner.test();

    let source2 = source::StringSource::new("HejHoppFastStäng".to_string());
    let scanner2 = scanner::Scanner::new(source2);
    scanner2.test(); 
}
