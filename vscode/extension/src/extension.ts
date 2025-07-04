import { GetOptionsPreferences, OptionsProvider } from '@optify/config';
import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import { ConfigParser } from './configparser';
import { PreviewBuilder, PreviewWhileEditingOptions } from './preview';

const outputChannel = vscode.window.createOutputChannel('Optify');

export function buildOptifyPreview(canonicalFeatures: string[], optifyRoot: string, editingOptions: PreviewWhileEditingOptions | undefined = undefined): string {
	console.debug(`Building preview for '${canonicalFeatures}' in '${optifyRoot}'`);
	const previewBuilder = new PreviewBuilder();
	try {
		// If some of the next lines fail in Rust from an unwrap or expect, then the exception is not caught.
		const provider = OptionsProvider.build(optifyRoot);
		const preferences = new GetOptionsPreferences();
		preferences.setSkipFeatureNameConversion(true);
		if (editingOptions?.overrides) {
			preferences.setOverridesJson(editingOptions.overrides);
		}
		const builtConfigJson = provider.getAllOptionsJson(editingOptions?.features ?? canonicalFeatures, preferences);
		const builtConfig = JSON.parse(builtConfigJson);
		return previewBuilder.getPreviewHtml(canonicalFeatures, builtConfig);
	} catch (error) {
		const message = `Failed to build preview${editingOptions ? " while editing" : ""}: ${error}`;
		console.error(message);
		if (!editingOptions) {
			vscode.window.showErrorMessage(message);
		}
		outputChannel.appendLine(message);
		const errorMessage = `${error}` + (editingOptions ? "\n\nIf the file was saved with any issues, then correct any issues and save the file to fix the preview." : "");
		return previewBuilder.getErrorPreviewHtml(canonicalFeatures, errorMessage);
	}
}

export function activate(context: vscode.ExtensionContext) {
	outputChannel.appendLine('Optify extension is now active!');

	// Set up context for when clauses
	updateOptifyFileContext();

	// Store active preview panels, their watchers, and document change listeners
	const activePreviews = new Map<string, { panel: vscode.WebviewPanel, watcher: vscode.FileSystemWatcher, documentChangeListener: vscode.Disposable }>();

	const previewCommand = vscode.commands.registerCommand('optify.previewFeature', async () => {
		const activeEditor = vscode.window.activeTextEditor;
		if (!activeEditor) {
			vscode.window.showErrorMessage('No active editor found');
			return;
		}

		const document = activeEditor.document;
		const filePath = document.fileName;

		outputChannel.appendLine(`Trying to get preview for file: ${filePath}`);

		try {
			const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
			if (!workspaceFolder) {
				vscode.window.showErrorMessage("File must be in a workspace");
				return;
			}

			const optifyRoot = findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
			if (!optifyRoot) {
				vscode.window.showErrorMessage("Could not find Optify root directory");
				return;
			}

			if (!isOptifyFeatureFile(filePath, optifyRoot, workspaceFolder)) {
				vscode.window.showErrorMessage("Current file is not an Optify feature file");
				return;
			}

			const canonicalName = getCanonicalName(filePath, optifyRoot);

			// Check if preview already exists
			const existingPreview = activePreviews.get(filePath);
			if (existingPreview) {
				existingPreview.panel.reveal();
				return;
			}

			const panel = vscode.window.createWebviewPanel(
				'optifyPreview',
				`Optify Preview: ${canonicalName}`,
				vscode.ViewColumn.Beside,
				{ enableScripts: false }
			);

			panel.webview.html = buildOptifyPreview([canonicalName], optifyRoot);

			// Create file watcher for this file
			const watcher = vscode.workspace.createFileSystemWatcher(filePath);

			// Update preview on file change
			const updatePreview = async () => {
				console.debug(`File changed: '${filePath}'. Remaking preview.`);
				panel.webview.html = buildOptifyPreview([canonicalName], optifyRoot);
			};

			watcher.onDidChange(updatePreview);

			// Also update preview on document text changes (before save)
			const documentChangeListener = vscode.workspace.onDidChangeTextDocument((event) => {
				if (event.document.uri.fsPath === filePath) {
					console.debug(`Document changed (unsaved): '${filePath}'. Updating preview.`);
					const documentText = event.document.getText();
					const config = ConfigParser.parse(documentText, event.document.languageId);
					panel.webview.html = buildOptifyPreview([canonicalName], optifyRoot, {
						features: config.imports ?? [],
						overrides: config.options ? JSON.stringify(config.options) : undefined
					});
				}
			});

			context.subscriptions.push(documentChangeListener);

			activePreviews.set(filePath, { panel, watcher, documentChangeListener });

			// Clean up when panel is closed
			panel.onDidDispose(() => {
				const preview = activePreviews.get(filePath);
				if (preview) {
					preview.watcher.dispose();
					preview.documentChangeListener.dispose();
					activePreviews.delete(filePath);
				}
			});
		} catch (error) {
			const errorMessage = `Error building Optify preview: ${error}`;
			console.error(errorMessage);
			outputChannel.appendLine(errorMessage);
			outputChannel.show();
			vscode.window.showErrorMessage(errorMessage);
		}
	});

	const documentLinkProvider = new OptifyDocumentLinkProvider();
	const linkProvider = vscode.languages.registerDocumentLinkProvider(
		[{ scheme: 'file', pattern: '**/*.{json,yaml,yml,json5}' }],
		documentLinkProvider
	);

	const definitionProvider = vscode.languages.registerDefinitionProvider(
		[{ scheme: 'file', pattern: '**/*.{json,yaml,yml,json5}' }],
		new OptifyDefinitionProvider()
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
		outputChannel,
		previewCommand,
		linkProvider,
		definitionProvider,
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

function isOptifyFeatureFile(filePath: string,
	optifyRoot: string | undefined = undefined,
	workspaceFolder: vscode.WorkspaceFolder | undefined = undefined): boolean {
	const ext = path.extname(filePath).toLowerCase();
	// We only support a few types of files in this extension and the config Rust crate only supports a few file types.
	if (!['.json', '.yaml', '.yml', '.json5'].includes(ext)) {
		return false;
	}

	// Check if file is in an Optify project by looking for root directory.
	if (!optifyRoot) {
		workspaceFolder ||= vscode.workspace.getWorkspaceFolder(vscode.Uri.file(filePath));
		if (!workspaceFolder) {
			return false;
		}

		optifyRoot = findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
	}
	return optifyRoot !== undefined;
}

export function findOptifyRoot(filePath: string, workspaceRoot: string): string | undefined {
	let currentDir = path.dirname(filePath);

	const configDirs = new Set(['options', 'configs', 'configurations']);
	const markerDirName = '.optify';
	while (currentDir !== path.dirname(currentDir)) {
		const currentDirName = path.basename(currentDir);
		if (configDirs.has(currentDirName)) {
			return currentDir;
		}

		const optifyConfigPath = path.join(currentDir, markerDirName);
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

		outputChannel.appendLine(`Providing document links for ${document.fileName} | languageId: ${document.languageId}`);
		const text = document.getText();
		const imports = ConfigParser.parseImports(text, document.languageId);
		if (imports) {
			for (let i = 0; i < imports.length; i++) {
				const importName = imports[i];
				const range = ConfigParser.findImportRange(text, importName, i, document.languageId);
				if (range) {
					const targetPath = resolveImportPath(importName, optifyRoot);
					if (targetPath) {
						const link = new vscode.DocumentLink(range, vscode.Uri.file(targetPath));
						links.push(link);
					}
				}
			}
		}

		return links;
	}
}

function resolveImportPath(importName: string, optifyRoot: string): string | undefined {
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

/**
 * Provides "Go to Definition" functionality for imports.
 */
class OptifyDefinitionProvider implements vscode.DefinitionProvider {
	provideDefinition(
		document: vscode.TextDocument,
		position: vscode.Position,
		_token: vscode.CancellationToken
	): vscode.ProviderResult<vscode.Definition> {
		const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
		if (!workspaceFolder) {
			return null;
		}

		const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);
		if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot)) {
			return null;
		}

		const text = document.getText();
		const imports = ConfigParser.parseImports(text, document.languageId);
		if (!imports) {
			return null;
		}

		// Check if the cursor is on an import
		for (let i = 0; i < imports.length; i++) {
			const importName = imports[i];
			const range = ConfigParser.findImportRange(text, importName, i, document.languageId);
			if (range && range.contains(position)) {
				const targetPath = resolveImportPath(importName, optifyRoot);
				if (targetPath) {
					return new vscode.Location(vscode.Uri.file(targetPath), new vscode.Position(0, 0));
				}
			}
		}

		return null;
	}
}

/**
 * Validates files such as validating imports.
 */
class OptifyDiagnosticsProvider {
	constructor(private diagnosticCollection: vscode.DiagnosticCollection) { }

	updateDiagnostics(document: vscode.TextDocument): void {
		outputChannel.appendLine(`Updating diagnostics for ${document.fileName} | languageId: ${document.languageId}`);
		const diagnostics: vscode.Diagnostic[] = [];
		const text = document.getText();

		const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
		if (!workspaceFolder) {
			return;
		}

		const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);
		if (!optifyRoot) {
			return;
		}

		// Get the canonical name of the current file for self-import detection
		const currentFileCanonicalName = getCanonicalName(document.uri.fsPath, optifyRoot);

		const imports = ConfigParser.parseImports(text, document.languageId);
		if (imports) {
			for (let i = 0; i < imports.length; i++) {
				const importName = imports[i];

				const targetPath = resolveImportPath(importName, optifyRoot);
				if (!targetPath) {
					const range = ConfigParser.findImportRange(text, importName, i, document.languageId);
					if (range) {
						const diagnostic = new vscode.Diagnostic(
							range,
							`Cannot resolve import '${importName}'`,
							vscode.DiagnosticSeverity.Error
						);
						diagnostics.push(diagnostic);
					}
				} else if (importName === currentFileCanonicalName) {
					const range = ConfigParser.findImportRange(text, importName, i, document.languageId);
					if (range) {
						const diagnostic = new vscode.Diagnostic(
							range,
							"A file cannot import itself",
							vscode.DiagnosticSeverity.Error
						);
						diagnostics.push(diagnostic);
					}
					continue;
				}

			}
		}

		this.diagnosticCollection.set(document.uri, diagnostics);
	}
}

export function deactivate() { }
