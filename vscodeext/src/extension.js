const vscode = require("vscode");
const vscodelc = require("vscode-languageclient/node");

const path = require("path");
const fs = require('fs');

module.exports = {
	activate,
	deactivate,
};

let client = null;
let generation = 1;

function activate(context) {
	console.log("Fudge Language Support extension activated");

	const traceOutputChannel = vscode.window.createOutputChannel("Fudge Language Server trace");

	// TODO!
	let serverPath = path.join('/Users', 'joel', 'Projects', 'fudgelang_rust', 'target', 'debug', 'fudgelsp');

	// Executable
	const run = {
		command: serverPath,
		options: {
			env: {
				...process.env,
				RUST_LOG: "debug",
			},
		},
	};

	// ServerOptions
	let serverOptions = {
		run: run,
		debug: run
	};

	// LanguageClientOptions
	let languageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [{ scheme: "file", language: "fudge" }],
		traceOutputChannel,
	};

	// Create the language client
	client = new vscodelc.LanguageClient("fudge-language-server", "Fudge Language Server", serverOptions, languageClientOptions);
	client.start();

	// Reload the server if the server binary changes
	fs.watchFile(serverPath, (curr, prev) => {
		console.log(`${serverPath} file changed! Restarting client...`);

		client.shutdown('stop', 10000).then(() => {
			console.log(`${serverPath} file changed! Restarting client...`);
			client = new vscodelc.LanguageClient("fudge-language-server-v" + generation, "Fudge Language Server", serverOptions, languageClientOptions);
			generation += 1;
			client.start();
		});
	});
}


function deactivate() {
	console.log("Fudge Language Support extension de-activated");

	client.stop();
}
