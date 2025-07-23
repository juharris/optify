import { GetOptionsPreferences } from '@optify/config';
import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import { ConfigParser } from './configparser';
import { PreviewBuilder, PreviewWhileEditingOptions } from './preview';
import { OptifyCompletionProvider } from './completion';
import { findOptifyRoot, isOptifyFeatureFile, getCanonicalName } from './path-utils';
import { getOptionsProvider, clearProviderCache, registerUpdateCallback } from './providers';


const outputChannel = vscode.window.createOutputChannel('Optify');

interface ActivePreview {
	panel: vscode.WebviewPanel
	documentChangeListener: vscode.Disposable
	debounceTimer?: NodeJS.Timeout
	updatePreview: () => void
}

const activePreviews = new Map<string, ActivePreview>();

export function buildOptifyPreview(canonicalFeatures: string[], optifyRoot: string, editingOptions: PreviewWhileEditingOptions | undefined = undefined): string {
	// console.debug(`Building preview for '${canonicalFeatures}' in '${optifyRoot}'`);
	const previewBuilder = new PreviewBuilder();
	try {
		// If some of the next lines fail in Rust from an unwrap or expect, then the exception is not caught.
		const provider = getOptionsProvider(optifyRoot);
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

	const EDIT_DEBOUNCE_MILLISECONDS = 250;

	// Generate all printable ASCII characters as trigger characters for completions
	const COMPLETION_TRIGGER_CHARACTERS: string[] = (() => {
		const triggers: string[] = [];
		// Add quotes and special characters
		triggers.push('"', "'", ' ', '-', '_', '/', '.', ':');
		// Add all letters (a-z, A-Z)
		for (let i = 65; i <= 90; i++) {
			triggers.push(String.fromCharCode(i)); // A-Z
			triggers.push(String.fromCharCode(i + 32)); // a-z
		}
		// Add all digits (0-9)
		for (let i = 48; i <= 57; i++) {
			triggers.push(String.fromCharCode(i));
		}
		return triggers;
	})();

	// Set up context for when clauses
	updateOptifyFileContext();

	// Register callback to update previews when options change
	registerUpdateCallback(() => {
		for (const preview of activePreviews.values()) {
			preview.updatePreview();
		}
	});

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

			const existingPreview = activePreviews.get(filePath);
			if (existingPreview) {
				existingPreview.panel.reveal();
				return;
			}

			const panel = vscode.window.createWebviewPanel(
				'optifyPreview',
				`Optify Preview: ${canonicalName}`,
				vscode.ViewColumn.Beside,
				{
					enableScripts: false,
					enableFindWidget: true
				}
			);

			panel.webview.html = buildOptifyPreview([canonicalName], optifyRoot);

			// Update the preview when the changes to the file are saved.
			const updatePreview = () => {
				// console.debug(`File changed: '${filePath}'. Remaking preview.`);
				panel.webview.html = buildOptifyPreview([canonicalName], optifyRoot);
			};

			// Update preview on document text changes (before save)
			const documentChangeListener = vscode.workspace.onDidChangeTextDocument((event) => {
				if (event.document.uri.fsPath === filePath) {
					const preview = activePreviews.get(filePath);
					if (!preview) {
						return;
					}

					if (preview.debounceTimer) {
						clearTimeout(preview.debounceTimer);
					}

					preview.debounceTimer = setTimeout(() => {
						// console.debug(`Document changed (unsaved): '${filePath}'. Updating preview.`);
						const documentText = event.document.getText();
						const config = ConfigParser.parse(documentText, event.document.languageId);
						panel.webview.html = buildOptifyPreview([canonicalName], optifyRoot, {
							features: config.imports ?? [],
							overrides: config.options ? JSON.stringify(config.options) : undefined
						});
					}, EDIT_DEBOUNCE_MILLISECONDS);
				}
			});

			context.subscriptions.push(documentChangeListener);

			activePreviews.set(filePath, { panel, documentChangeListener, updatePreview, });

			// Clean up when panel is closed
			panel.onDidDispose(() => {
				cleanPreview(filePath);
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

	const completionProvider = vscode.languages.registerCompletionItemProvider(
		[{ scheme: 'file', pattern: '**/*.{json,yaml,yml,json5}' }],
		new OptifyCompletionProvider(outputChannel),
		...COMPLETION_TRIGGER_CHARACTERS
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
		completionProvider,
		diagnosticCollection,
		onDidChangeDocument,
		onDidOpenDocument,
		onDidChangeActiveEditor
	);
}

function cleanPreview(filePath: string) {
	const preview = activePreviews.get(filePath);
	if (preview) {
		preview.documentChangeListener.dispose();
		// Clear any pending debounce timer
		if (preview.debounceTimer) {
			clearTimeout(preview.debounceTimer);
		}
		activePreviews.delete(filePath);
	}
}

function updateOptifyFileContext() {
	const activeEditor = vscode.window.activeTextEditor;
	const isOptifyFile = activeEditor ? isOptifyFeatureFile(activeEditor.document.fileName) : false;
	vscode.commands.executeCommand('setContext', 'optify.isOptifyFile', isOptifyFile);
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
		const importInfos = ConfigParser.findImportRanges(text, document.languageId);

		for (const importInfo of importInfos) {
			const targetPath = resolveImportPath(importInfo.name, optifyRoot);
			if (targetPath) {
				const link = new vscode.DocumentLink(importInfo.range, vscode.Uri.file(targetPath));
				links.push(link);
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
		const importInfos = ConfigParser.findImportRanges(text, document.languageId);

		// Check if the cursor is on an import
		for (const importInfo of importInfos) {
			if (importInfo.range.contains(position)) {
				const targetPath = resolveImportPath(importInfo.name, optifyRoot);
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

		const importInfos = ConfigParser.findImportRanges(text, document.languageId);
		for (const importInfo of importInfos) {
			const targetPath = resolveImportPath(importInfo.name, optifyRoot);
			if (!targetPath) {
				const diagnostic = new vscode.Diagnostic(
					importInfo.range,
					`Cannot resolve import '${importInfo.name}'`,
					vscode.DiagnosticSeverity.Error
				);
				diagnostics.push(diagnostic);
			} else if (importInfo.name === currentFileCanonicalName) {
				const diagnostic = new vscode.Diagnostic(
					importInfo.range,
					"A file cannot import itself",
					vscode.DiagnosticSeverity.Error
				);
				diagnostics.push(diagnostic);
			}
		}

		this.diagnosticCollection.set(document.uri, diagnostics);
	}
}

export function deactivate() {
	console.debug("Deactivating Optify extension");
	for (const filePath of activePreviews.keys()) {
		cleanPreview(filePath);
	}
	// It should already be empty, but we'll clear it just in case.
	activePreviews.clear();
	clearProviderCache();
}
