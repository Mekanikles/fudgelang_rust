use crate::error::*;
use crate::source::*;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{BufferedStandardStream, ColorChoice};

pub fn print_errors<'a>(errors: &Vec<Error>, source: &'a Source) {
    let mut files = SimpleFiles::new();
    let file_id = files.add(source.name(), source.to_str());

    for err in errors {
        let diagnostic = Diagnostic::error()
            .with_message(err.message.clone())
            .with_code(error_code(err.id))
            .with_labels(vec![Label::primary(
                file_id,
                err.source_span.pos as usize..(err.source_span.pos as usize + err.source_span.len),
            )]);

        // Output error
        let mut writer = BufferedStandardStream::stdout(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();
        let _ = codespan_reporting::term::emit(&mut writer, &config, &files, &diagnostic);

        // Output backtrace
        // TODO: Only in compiler dev mode
        if let Some(bt) = &err.backtrace {
            let output = format!("{:?}", bt);
            let filtered: Vec<&str> = output
                .lines()
                .filter(|i| i.contains("libfudgec"))
                .filter(|i| !i.contains("ErrorManager"))
                .filter(|i| !i.is_empty())
                .collect();

            println!("ERROR BACKTRACE:");
            for l in filtered {
                println!("{}", l);
            }
        }
    }
}
