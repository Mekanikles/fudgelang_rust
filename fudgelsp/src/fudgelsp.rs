use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::parser::tokenstream::TokenStream;
use libfudgec::*;

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::LOG, "Fudge Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::LOG, "Fudge Language Server shutting down")
            .await;
        Ok(())
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let msg = format!("File changed!");
        self.client
            .log_message(MessageType::LOG, msg.to_string())
            .await;

        let mut output = Vec::new();
        for change in params.content_changes {
            let rope = ropey::Rope::from_str(&change.text);
            let source = source::Source::from_str(&change.text);
            let scanner_result = scanner::tokenize(&source);
            let parser_result =
                parser::parse(&mut TokenStream::new(&scanner_result.tokens, &source));

            for error in parser_result
                .errors
                .iter()
                .chain(scanner_result.errors.iter())
            {
                let pos1 = error.source_span.pos;
                let pos2 = pos1 + error.source_span.len as u64;
                output.push(Diagnostic::new_simple(
                    Range::new(
                        rope.offset_to_position(pos1 as usize),
                        rope.offset_to_position(pos2 as usize),
                    ),
                    error.message.clone(), // Bah
                ));
            }
        }

        self.client
            .publish_diagnostics(
                params.text_document.uri.clone(),
                output,
                Some(params.text_document.version),
            )
            .await;
    }
}

trait RopeExt {
    fn offset_to_position(&self, offset: usize) -> Position;
}

impl RopeExt for ropey::Rope {
    fn offset_to_position(&self, offset: usize) -> Position {
        let line = self.char_to_line(offset);
        let first_char = self.line_to_char(line);
        let column = offset - first_char;
        return Position::new(line as u32, column as u32);
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
