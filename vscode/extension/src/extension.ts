import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as yaml from 'js-yaml';
import { OptionsProvider } from '@optify/config';

export function activate(context: vscode.ExtensionContext) {
	console.log('Optify extension is now active!');

	const previewCommand = vscode.commands.registerCommand('optify.previewFeature', async () => {
		console.log(`[Optify] Debug: previewFeature command triggered`);
		console.debug("[Optify] Debug test: previewFeature command triggered");

		const activeEditor = vscode.window.activeTextEditor;
		if (!activeEditor) {
			vscode.window.showErrorMessage('No active editor found');
			return;
		}

		const document = activeEditor.document;
		const filePath = document.fileName;

		console.log(`[Optify] Debug: filePath=${filePath}`);
		console.debug("[Optify] Debug test");

		if (!isOptifyFeatureFile(filePath)) {
			vscode.window.showErrorMessage('Current file is not an Optify feature file');
			return;
		}

		try {
			const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
			if (!workspaceFolder) {
				vscode.window.showErrorMessage('File must be in a workspace');
				return;
			}

			const optifyRoot = findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
			if (!optifyRoot) {
				vscode.window.showErrorMessage('Could not find Optify root directory');
				return;
			}

			const canonicalName = getCanonicalName(filePath, optifyRoot);

			const provider = OptionsProvider.build(optifyRoot);
			const builtConfigJson = provider.getAllOptionsJson([canonicalName]);
			const builtConfig = JSON.parse(builtConfigJson);

			const panel = vscode.window.createWebviewPanel(
				'optifyPreview',
				`Optify Preview: ${canonicalName}`,
				vscode.ViewColumn.Beside,
				{ enableScripts: false }
			);

			panel.webview.html = getPreviewHtml(canonicalName, builtConfig);
		} catch (error) {
			console.error('Optify build error:', error);
			vscode.window.showErrorMessage(`Failed to build feature: ${error}`);
		}
	});

	const documentLinkProvider = new OptifyDocumentLinkProvider();
	const linkProvider = vscode.languages.registerDocumentLinkProvider(
		[{ scheme: 'file' }],
		documentLinkProvider
	);

	const diagnosticCollection = vscode.languages.createDiagnosticCollection('optify');
	const diagnosticsProvider = new OptifyDiagnosticsProvider(diagnosticCollection);

	const onDidChangeDocument = vscode.workspace.onDidChangeTextDocument((event) => {
		if (isOptifyFeatureFile(event.document.fileName)) {
			diagnosticsProvider.updateDiagnostics(event.document);
		}
	});

	const onDidOpenDocument = vscode.workspace.onDidOpenTextDocument((document) => {
		if (isOptifyFeatureFile(document.fileName)) {
			diagnosticsProvider.updateDiagnostics(document);
		}
	});

	context.subscriptions.push(
		previewCommand,
		linkProvider,
		diagnosticCollection,
		onDidChangeDocument,
		onDidOpenDocument
	);
}

function isOptifyFeatureFile(filePath: string): boolean {
	const ext = path.extname(filePath).toLowerCase();
	// We only support a few types of files in this extension and the config Rust crate only supports a few file types.
	if (!['.json', '.yaml', '.yml', '.json5'].includes(ext)) {
		return false;
	}

	// Check if file is in an Optify project by looking for root directory
	const workspaceFolder = vscode.workspace.getWorkspaceFolder(vscode.Uri.file(filePath));
	if (!workspaceFolder) {
		return false;
	}

	const optifyRoot = findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
	return optifyRoot !== undefined;
}

function findOptifyRoot(filePath: string, workspaceRoot: string): string | undefined {
	let currentDir = path.dirname(filePath);

	const configDirs = ['options', 'configs', 'configurations'];
	while (currentDir !== workspaceRoot && currentDir !== path.dirname(currentDir)) {
		// Check for configuration directories
		for (const configDir of configDirs) {
			const configPath = path.join(currentDir, configDir);
			if (fs.existsSync(configPath) && fs.statSync(configPath).isDirectory()) {
				return configPath;
			}
		}

		// Check for .optify.yaml with root: true
		const optifyConfigPath = path.join(currentDir, '.optify.yaml');
		if (fs.existsSync(optifyConfigPath)) {
			try {
				const content = fs.readFileSync(optifyConfigPath, 'utf8');
				const config = yaml.load(content) as any;
				if (config && config.root === true) {
					return currentDir;
				}
			} catch (error) {
				// Ignore YAML parse errors
			}
		}

		currentDir = path.dirname(currentDir);
	}

	// If we reach the workspace root, check if it contains config directories
	for (const configDir of configDirs) {
		const configPath = path.join(workspaceRoot, configDir);
		if (fs.existsSync(configPath) && fs.statSync(configPath).isDirectory()) {
			return configPath;
		}
	}

	return undefined;
}

function getCanonicalName(filePath: string, optifyRoot: string): string {
	const relativePath = path.relative(optifyRoot, filePath);
	const result = path.join(path.dirname(relativePath), path.basename(relativePath, path.extname(relativePath)));

	return result;
}

function getPreviewHtml(canonicalName: string, config: any): string {
	const configJson = JSON.stringify(config, null, 2);
	return `
		<!DOCTYPE html>
		<html>
		<head>
			<title>Optify Preview: ${canonicalName}</title>
			<style>
				body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; }
				pre { background: #f5f5f5; padding: 15px; border-radius: 5px; overflow-x: auto; }
				h1 { color: #333; border-bottom: 2px solid #007acc; padding-bottom: 10px; }
			</style>
		</head>
		<body>
			<h1>Features: [${canonicalName}]</h1>
			<pre><code>${configJson}</code></pre>
		</body>
		</html>
	`;
}

class OptifyDocumentLinkProvider implements vscode.DocumentLinkProvider {
	provideDocumentLinks(document: vscode.TextDocument): vscode.DocumentLink[] {
		const links: vscode.DocumentLink[] = [];

		// Only provide links for Optify feature files
		if (!isOptifyFeatureFile(document.fileName)) {
			return links;
		}

		const text = document.getText();

		try {
			const config = JSON.parse(text);
			if (config.imports && Array.isArray(config.imports)) {
				const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
				if (!workspaceFolder) {
					return links;
				}

				for (let i = 0; i < config.imports.length; i++) {
					const importName = config.imports[i];
					if (typeof importName === 'string') {
						const range = this.findImportRange(text, importName, i);
						if (range) {
							const targetPath = this.resolveImportPath(importName, document.uri.fsPath, workspaceFolder.uri.fsPath);
							if (targetPath) {
								const link = new vscode.DocumentLink(range, vscode.Uri.file(targetPath));
								links.push(link);
							}
						}
					}
				}
			}
		} catch (error) {
			// Ignore JSON parse errors
		}

		return links;
	}

	private findImportRange(text: string, importName: string, index: number): vscode.Range | undefined {
		const importsMatch = text.match(/"imports"\s*:\s*\[([^\]]*)\]/s);
		if (!importsMatch) {
			return undefined;
		}

		const importsContent = importsMatch[1];
		const importPattern = new RegExp(`"${importName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}"`, 'g');
		let match;
		let currentIndex = 0;

		while ((match = importPattern.exec(importsContent)) !== null) {
			if (currentIndex === index) {
				const startPos = importsMatch.index! + importsMatch[0].indexOf(importsMatch[1]) + match.index! + 1;
				const endPos = startPos + importName.length;

				const startPosition = this.getPositionFromIndex(text, startPos);
				const endPosition = this.getPositionFromIndex(text, endPos);

				return new vscode.Range(startPosition, endPosition);
			}
			currentIndex++;
		}

		return undefined;
	}

	private getPositionFromIndex(text: string, index: number): vscode.Position {
		const lines = text.substring(0, index).split('\n');
		return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
	}

	private resolveImportPath(importName: string, currentFilePath: string, workspaceRoot: string): string | undefined {
		const optifyRoot = findOptifyRoot(currentFilePath, workspaceRoot);
		if (!optifyRoot) {
			return undefined;
		}

		const extensions = ['.json', '.yaml', '.yml', '.json5'];

		// Try resolving relative to the optify root
		for (const ext of extensions) {
			const possiblePath = path.resolve(optifyRoot, importName + ext);
			if (fs.existsSync(possiblePath)) {
				return possiblePath;
			}
		}

		return undefined;
	}
}

class OptifyDiagnosticsProvider {
	constructor(private diagnosticCollection: vscode.DiagnosticCollection) { }

	updateDiagnostics(document: vscode.TextDocument): void {
		const diagnostics: vscode.Diagnostic[] = [];

		// Only provide diagnostics for Optify feature files
		if (!isOptifyFeatureFile(document.fileName)) {
			this.diagnosticCollection.set(document.uri, diagnostics);
			return;
		}

		const text = document.getText();

		try {
			const config = JSON.parse(text);
			if (config.imports && Array.isArray(config.imports)) {
				const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
				if (!workspaceFolder) {
					return;
				}

				for (let i = 0; i < config.imports.length; i++) {
					const importName = config.imports[i];
					if (typeof importName === 'string') {
						const targetPath = this.resolveImportPath(importName, document.uri.fsPath, workspaceFolder.uri.fsPath);
						if (!targetPath) {
							const range = this.findImportRange(text, importName, i);
							if (range) {
								const diagnostic = new vscode.Diagnostic(
									range,
									`Cannot resolve import '${importName}'`,
									vscode.DiagnosticSeverity.Error
								);
								diagnostics.push(diagnostic);
							}
						}
					}
				}
			}
		} catch (error) {
			// Ignore JSON parse errors as they're handled by JSON language server
		}

		this.diagnosticCollection.set(document.uri, diagnostics);
	}

	private findImportRange(text: string, importName: string, index: number): vscode.Range | undefined {
		const importsMatch = text.match(/"imports"\s*:\s*\[([^\]]*)\]/s);
		if (!importsMatch) {
			return undefined;
		}

		const importsContent = importsMatch[1];
		const importPattern = new RegExp(`"${importName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}"`, 'g');
		let match;
		let currentIndex = 0;

		while ((match = importPattern.exec(importsContent)) !== null) {
			if (currentIndex === index) {
				const startPos = importsMatch.index! + importsMatch[0].indexOf(importsMatch[1]) + match.index! + 1;
				const endPos = startPos + importName.length;

				const startPosition = this.getPositionFromIndex(text, startPos);
				const endPosition = this.getPositionFromIndex(text, endPos);

				return new vscode.Range(startPosition, endPosition);
			}
			currentIndex++;
		}

		return undefined;
	}

	private getPositionFromIndex(text: string, index: number): vscode.Position {
		const lines = text.substring(0, index).split('\n');
		return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
	}

	private resolveImportPath(importName: string, currentFilePath: string, workspaceRoot: string): string | undefined {
		const optifyRoot = findOptifyRoot(currentFilePath, workspaceRoot);
		if (!optifyRoot) {
			return undefined;
		}

		const extensions = ['.json', '.yaml', '.yml', '.json5'];

		// Try resolving relative to the optify root
		for (const ext of extensions) {
			const possiblePath = path.resolve(optifyRoot, importName + ext);
			if (fs.existsSync(possiblePath)) {
				return possiblePath;
			}
		}

		return undefined;
	}
}

export function deactivate() { }
