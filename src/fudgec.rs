#[macro_use]
extern crate tempus_fugit;

use libfudgec::*;

use scanner::Scanner;
use structopt::StructOpt;

use ansi_term::Colour as Color;
use ansi_term::Style;

#[derive(StructOpt)]
struct CommandLineParameters {
    // Path to file
    #[structopt(parse(from_os_str))]
    file: std::path::PathBuf,

    #[structopt(short = "r", long = "repeats", default_value = "1")]
    repeats: u64,

    #[structopt(short = "t", long = "print-tokens")]
    print_tokens: bool,
}

fn main() {
    let params = CommandLineParameters::from_args();

    let repeats = params.repeats;
    let source = source::FileSource::from_filepath(params.file.clone());

    let mut tokens = Vec::with_capacity(100000);

    let mut total_time = tempus_fugit::Measurement::zero();

    for _i in 0..repeats {
        // Scan all tokens
        tokens.clear();
        let mut scanner = scanner::ScannerImpl::new(&source);
        let (_, measurement) = measure! {{
            while let Some(n) = scanner.read_token() {
                tokens.push(n);
            }
        }};

        total_time = (total_time + measurement).unwrap();

        if params.print_tokens {
            println!("Tokens:");
            print!("    ");
            for t in &tokens {
                print!(
                    "{:?}, ",
                    scanner::token::TokenDisplay {
                        token: t,
                        scanner: &scanner
                    }
                );
            }
            println!("");
        }

        // Print errors
        for err in &scanner.errors {
            // Draw header
            let lineinfo = scanner.get_line_info(err.source_span.pos).unwrap();
            let error_label = Color::Red.bold().paint("Error:");
            let error_message = Style::new().bold().paint(err.message.clone());
            println!("{label} {message} ({file}:{row}:{column})", 
                label = error_label, 
                message = error_message, 
                file = params.file.to_str().unwrap(), 
                row = lineinfo.row, 
                column = lineinfo.column);

            let mut span_start = err.source_span.pos as usize - lineinfo.line_start;
            let mut span_end = span_start + err.source_span.len as usize;

            // Check how may tabs will be converted to spaces before and mid-span
            let pre_span_tabs = lineinfo.text[..span_start].matches('\t').count();
            let in_span_tabs = lineinfo.text[span_start..span_end].matches('\t').count();

            // Replace tabs with spaces for consistent output
            let trimmed_tab_replaced_text = lineinfo.text.trim_end().replace('\t', "    ");

            // Adjust span for tabs
            span_start += pre_span_tabs * 3;
            span_end += pre_span_tabs * 3 + in_span_tabs * 3;

            // Draw code line
            let row_header = Color::Blue.bold().paint(format!(" {} | ", lineinfo.row));
            println!("{}{}", row_header, trimmed_tab_replaced_text);

            // Draw squiggles
            let span_start_char_count = trimmed_tab_replaced_text[..span_start].chars().count();
            let span_len_char_count = trimmed_tab_replaced_text[span_start..span_end].chars().count();
            let squiggles = format!("{}", Color::Red.bold().paint("^".repeat(span_len_char_count)));
            println!("{}{}{}", " ".repeat(row_header.len()), 
                " ".repeat(span_start_char_count), 
                squiggles);
        }
    }

    println!(
        "Scanned {} tokens in {}, {} times. ({} per scan)",
        tokens.len(),
        total_time,
        repeats,
        tempus_fugit::Measurement::from(tempus_fugit::Duration::from(total_time.clone()) / repeats as i32)
    );
}
