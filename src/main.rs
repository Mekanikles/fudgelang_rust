mod source;
mod scanner;

fn main() {
    println!("Scan from bytes:");
    let source = source::MemorySource::from_bytes("Hejhopp".as_bytes());
    let scanner = scanner::Scanner::new(source);
    scanner.test();

    println!("Scan from string:");
    let source = source::MemorySource::from_str("HejHoppFastStr");
    let scanner = scanner::Scanner::new(source);
    scanner.test();

    println!("Scan from file in memory:");
    let source = source::MemorySource::from_file("testdata/test.fu");
    let scanner = scanner::Scanner::new(source);
    scanner.test();

    println!("Scan from streamed file:");
    let source = source::FileSource::new("testdata/test.fu");
    let scanner = scanner::Scanner::new(source);
    scanner.test();
}
