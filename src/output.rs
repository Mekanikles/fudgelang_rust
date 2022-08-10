use crate::error::*;
use crate::source::*;

use std::io::Read;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

pub fn print_errors<'a, R: Read, S: Source<'a, R>>(errors: &Vec<Error>, source: &'a S) {
    // TODO: If we only use memory sources, we don't have to
    //  copy text here, we can implement the source/file traits used
    //  by codespan.
    let mut text: String = String::default();
    if source.get_readable().read_to_string(&mut text).is_ok() {
        let mut files = SimpleFiles::new();
        let file_id = files.add(source.get_name(), text);

        for err in errors {
            let diagnostic = Diagnostic::error()
                .with_message(err.message.clone())
                .with_code(error_code(err.id))
                .with_labels(vec![Label::primary(
                    file_id,
                    err.source_span.pos as usize
                        ..(err.source_span.pos as usize + err.source_span.len),
                )]);

            // Write to terminal
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();
            let _ =
                codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diagnostic);
        }
    }
}
