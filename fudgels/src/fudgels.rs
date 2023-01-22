use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::parser::tokenstream::TokenStream;
use dashmap::DashMap;
use libfudgec::*;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: DashMap<Url, Document>,
}

fn create_backend(client: Client) -> Backend {
    Backend {
        client: client,
        documents: DashMap::new(),
    }
}

#[derive(Debug)]
struct Document {
    name: String,
    rope: ropey::Rope,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        // TODO: Might have to sync already opened documents here

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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let msg = format!("======> File OPENED! Params: {:?}", params);
        self.client
            .log_message(MessageType::LOG, msg.to_string())
            .await;

        let rope = ropey::Rope::from_str(&params.text_document.text);
        let doc = Document {
            name: params.text_document.uri.to_string(),
            rope: rope,
        };

        let diags = Backend::generate_diagnostics(&doc);
        self.documents.insert(params.text_document.uri.clone(), doc);

        self.client
            .publish_diagnostics(
                params.text_document.uri,
                diags,
                Some(params.text_document.version),
            )
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let msg = format!("======> File CLOSED! Params: {:?}", params);
        self.client
            .log_message(MessageType::LOG, msg.to_string())
            .await;

        self.documents.remove(&params.text_document.uri);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let msg = format!("======> File CHANGED! Params: {:?}", params);
        self.client
            .log_message(MessageType::LOG, msg.to_string())
            .await;

        let mut doc = self.documents.get_mut(&params.text_document.uri).unwrap();
        for change in params.content_changes {
            let start_idx = doc
                .rope
                .line_to_char(change.range.unwrap().start.line as usize)
                + (change.range.unwrap().start.character as usize);
            let end_idx = doc
                .rope
                .line_to_char(change.range.unwrap().end.line as usize)
                + (change.range.unwrap().end.character as usize);

            // Remove the segment
            doc.rope.remove(start_idx..end_idx);

            // And replace it with something better.
            doc.rope.insert(start_idx, &change.text);
        }

        let msg = format!(
            "======> Complete doc after change: {:?}",
            doc.rope.to_string()
        );
        self.client
            .log_message(MessageType::LOG, msg.to_string())
            .await;

        let diags = Backend::generate_diagnostics(&doc);

        self.client
            .publish_diagnostics(
                params.text_document.uri.clone(),
                diags,
                Some(params.text_document.version),
            )
            .await;
    }
}

impl Backend {
    fn generate_diagnostics(document: &Document) -> Vec<Diagnostic> {
        let mut output = Vec::new();
        let rope = &document.rope;
        let source = source::Source::from_string(document.name.clone(), rope.to_string());
        let scanner_result = scanner::tokenize(&source);
        let parser_result = parser::parse(&mut TokenStream::new(&scanner_result.tokens, &source));

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
        return output;
    }
}

trait RopeExt {
    fn offset_to_position(&self, offset: usize) -> Position;
}

impl RopeExt for ropey::Rope {
    fn offset_to_position(&self, offset: usize) -> Position {
        let offset = std::cmp::min(offset, self.len_bytes());
        let char_pos = self.byte_to_char(offset);
        let line = self.char_to_line(char_pos);
        let first_char = self.line_to_char(line);
        let column = char_pos - first_char;
        return Position::new(line as u32, column as u32);
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| create_backend(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
