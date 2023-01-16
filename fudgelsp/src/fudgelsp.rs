use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

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
            .log_message(MessageType::INFO, "Fudge Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "Fudge Language Server shutting down")
            .await;
        Ok(())
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let msg = format!("File changed: {:?}", params);
        self.client
            .log_message(MessageType::INFO, msg.to_string())
            .await;

        let test = Diagnostic::new_simple(
            Range::new(Position::new(4, 3), Position::new(4, 7)),
            "Test error".into(),
        );

        self.client
            .publish_diagnostics(
                params.text_document.uri.clone(),
                Vec::from([test]),
                Some(params.text_document.version),
            )
            .await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
