const vscode = require("vscode");
const vscodelc = require("vscode-languageclient/node");

const path = require("path");

module.exports = {
	activate,
	deactivate,
};

function activate(context) {
	console.log("Fudge Language Support extension activated");

	const traceOutputChannel = vscode.window.createOutputChannel("Fudge Language Server trace");

	// TODO!
	let command = path.join('Users', 'joel', 'Projects', 'test', 'fudge-language-support', 'rustlsp', 'target', 'debug', 'rustlsp');

	// Executable
	const run = {
		command,
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

	// Add all disposables here
	context.subscriptions.push(client.start());
}

/*
  const traceOutputChannel = window.createOutputChannel("Nrs Language Server trace");
  const command = process.env.SERVER_PATH || "nrs-language-server";
  const run: Executable = {
	command,
	options: {
	  env: {
		...process.env,
		// eslint-disable-next-line @typescript-eslint/naming-convention
		RUST_LOG: "debug",
	  },
	},
  };
  const serverOptions: ServerOptions = {
	run,
	debug: run,
  };
  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  // Options to control the language client
  let clientOptions: LanguageClientOptions = {
	// Register the server for plain text documents
	documentSelector: [{ scheme: "file", language: "nrs" }],
	synchronize: {
	  // Notify the server about file changes to '.clientrc files contained in the workspace
	  */
//fileEvents: workspace.createFileSystemWatcher("**/.clientrc"),
/*
},
traceOutputChannel,
};

// Create the language client and start the client.
client = new LanguageClient("nrs-language-server", "nrs language server", serverOptions, clientOptions);
activateInlayHints(context);
client.start();*/

/*
export function activate(context: vscode.ExtensionContext) {

	// This line of code will only be executed once when your extension is activated

	// TODO: Start server exe and communicate with it
	let serverExe = <Path_to_server>;

	let ServerOptions: ServerOptions = {
		run: {command: serverExe, args:['-lsp']},
		debug: {command: serverExe, args:['-lsp']}
	}

	let clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [
			{*/
//pattern: '**/*.txt',
/*}
],

}

let lspClient = new LanguageClient("Hello LSP", ServerOptions, clientOptions);

// For debugging only
//lspClient.trace = Trace.Verbose;

//add all disposables here
context.subscriptions.push(lspClient.start());
}
*/


function sayHello() {
	vscode.window.showInformationMessage("Hello Poop!");
}

function deactivate() { }
