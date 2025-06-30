import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as yaml from 'js-yaml';
import { GetOptionsPreferences, OptionsProvider } from '@optify/config';

export function activate(context: vscode.ExtensionContext) {
	console.debug('Optify extension is now active!');

	// Set up context for when clauses
	updateOptifyFileContext();

	const previewCommand = vscode.commands.registerCommand('optify.previewFeature', async () => {
		const activeEditor = vscode.window.activeTextEditor;
		if (!activeEditor) {
			vscode.window.showErrorMessage('No active editor found');
			return;
		}

		const document = activeEditor.document;
		const filePath = document.fileName;

		console.debug(`Previewing feature file: ${filePath}`);
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
			const preferences = new GetOptionsPreferences();
			preferences.setSkipFeatureNameConversion(true);
			const builtConfigJson = provider.getAllOptionsJson([canonicalName], preferences);
			const builtConfig = JSON.parse(builtConfigJson);

			const panel = vscode.window.createWebviewPanel(
				'optifyPreview',
				`Optify Preview: ${canonicalName}`,
				vscode.ViewColumn.Beside,
				{ enableScripts: false }
			);

			panel.webview.html = getPreviewHtml([canonicalName], builtConfig);
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
		updateOptifyFileContext();
	});

	const onDidChangeActiveEditor = vscode.window.onDidChangeActiveTextEditor(() => {
		updateOptifyFileContext();
	});

	context.subscriptions.push(
		previewCommand,
		linkProvider,
		diagnosticCollection,
		onDidChangeDocument,
		onDidOpenDocument,
		onDidChangeActiveEditor
	);
}

function updateOptifyFileContext() {
	const activeEditor = vscode.window.activeTextEditor;
	const isOptifyFile = activeEditor ? isOptifyFeatureFile(activeEditor.document.fileName) : false;
	vscode.commands.executeCommand('setContext', 'optify.isOptifyFile', isOptifyFile);
}

function isOptifyFeatureFile(filePath: string, optifyRoot: string | undefined = undefined): boolean {
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

	optifyRoot ||= findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
	return optifyRoot !== undefined;
}

function findOptifyRoot(filePath: string, workspaceRoot: string): string | undefined {
	let currentDir = path.dirname(filePath);

	const configDirs = ['options', 'configs', 'configurations'];
	while (currentDir !== path.dirname(currentDir)) {
		// Check for configuration directories
		const currentDirName = path.basename(currentDir);
		if (configDirs.includes(currentDirName)) {
			return currentDir;
		}

		// Look for some kind of marker file, but it shouldn't be a suffix for a feature file.
		const optifyConfigPath = path.join(currentDir, '.optify');
		if (fs.existsSync(optifyConfigPath)) {
			return currentDir;
		}

		currentDir = path.dirname(currentDir);
		if (currentDir === workspaceRoot) {
			return undefined;
		}
	}

	return undefined;
}

function getCanonicalName(filePath: string, optifyRoot: string): string {
	const relativePath = path.relative(optifyRoot, filePath);
	const result = path.join(path.dirname(relativePath), path.basename(relativePath, path.extname(relativePath)));

	return result;
}

function getPreviewHtml(features: string[], config: any): string {
	const configJson = JSON.stringify(config, null, 2);
	const featuresString = JSON.stringify(features);
	return `
		<!DOCTYPE html>
		<html>
		<head>
			<title>Optify Preview: ${featuresString}</title>
			<style>
				body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; }
				h2 { border-bottom: 2px solid #007acc; padding-bottom: 10px; }
				pre { padding: 1rem; overflow-x: auto; background-color: #383838; color: #d8d8d8; border-radius: 4px; }
				code { background-color: transparent; font-family: 'Courier New', Courier, monospace; }
			</style>
		</head>
		<body>
			<h2>Features: <code>${featuresString}</code></h1>
			<pre><code>${configJson}</code></pre>
		</body>
		</html>
	`;
}

/**
 * Adds links to imports.
 */
class OptifyDocumentLinkProvider implements vscode.DocumentLinkProvider {
	provideDocumentLinks(document: vscode.TextDocument): vscode.DocumentLink[] {
		const links: vscode.DocumentLink[] = [];
		const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
		if (!workspaceFolder) {
			return links; // No workspace folder, no links
		}

		const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);

		// Only provide links for Optify feature files
		if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot)) {
			return links;
		}

		console.debug(`Providing document links for ${document.fileName} | languageId: ${document.languageId}`);
		const text = document.getText();

		switch (document.languageId) {
			case 'json':
				try {
					const config = JSON.parse(text);
					if (config.imports && Array.isArray(config.imports)) {
						if (!workspaceFolder) {
							return links;
						}
						for (let i = 0; i < config.imports.length; ++i) {
							const importName = config.imports[i];
							if (typeof importName === 'string') {
								const range = this.findImportRange(text, importName, i);
								if (range) {
									const targetPath = this.resolveImportPath(importName, optifyRoot);
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
			case 'yaml':
			// TODO
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
			++currentIndex;
		}

		return undefined;
	}

	private getPositionFromIndex(text: string, index: number): vscode.Position {
		const lines = text.substring(0, index).split('\n');
		return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
	}

	private resolveImportPath(importName: string, optifyRoot: string): string | undefined {
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

/**
 * Validates files such as validating imports.
 */
class OptifyDiagnosticsProvider {
	constructor(private diagnosticCollection: vscode.DiagnosticCollection) { }

	updateDiagnostics(document: vscode.TextDocument): void {
		console.debug(`Updating diagnostics for ${document.fileName} | languageId: ${document.languageId}`);
		const diagnostics: vscode.Diagnostic[] = [];
		const text = document.getText();

		switch (document.languageId) {
			case 'json':
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
			case 'yaml':
			// TODO
		}
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
