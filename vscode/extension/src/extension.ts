import { GetOptionsPreferences } from '@optify/config';
import * as vscode from 'vscode';
import { OptifyCompletionProvider } from './completion';
import { ConfigParser } from './config-parser';
import { OptifyDefinitionProvider } from './definitions';
import { OptifyDependentsHoverProvider } from './dependents/hover';
import { OptifyDependentsProvider } from './dependents/show';
import { OptifyCodeActionProvider, OptifyDiagnosticsProvider } from './diagnostics';
import { findOptifyRoot, getCanonicalName, isOptifyFeatureFile, resolveImportPath } from './path-utils';
import { PreviewBuilder, PreviewWhileEditingOptions } from './preview';
import { clearProviderCache, getOptionsProvider, registerUpdateCallback } from './providers';

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
		const feature = canonicalFeatures.length === 1 ? canonicalFeatures[0] : undefined;
		const dependents = feature ? provider.featuresWithMetadata()[feature]?.dependents() : null;
		return previewBuilder.getPreviewHtml(canonicalFeatures, builtConfig, dependents, provider);
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

	// Create the dependents providers
	const dependentsProvider = new OptifyDependentsProvider(outputChannel);
	const dependentsHoverProvider = new OptifyDependentsHoverProvider(outputChannel);

	// Register callback to update previews and dependents when options change
	registerUpdateCallback(() => {
		for (const preview of activePreviews.values()) {
			preview.updatePreview();
		}
		// Update dependents decoration for active editor
		if (vscode.window.activeTextEditor) {
			dependentsProvider.updateDependentsDecoration(vscode.window.activeTextEditor);
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
					enableScripts: true,
					enableFindWidget: true
				}
			);

			panel.webview.html = buildOptifyPreview([canonicalName], optifyRoot);

			// Handle messages from the webview
			panel.webview.onDidReceiveMessage(
				async (message) => {
					if (message.command === 'openFile') {
						if (message.path) {
							const uri = vscode.Uri.file(message.path);
							vscode.window.showTextDocument(uri);
						}
					}
				},
				undefined,
				context.subscriptions
			);

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

	const codeActionProvider = vscode.languages.registerCodeActionsProvider(
		[{ scheme: 'file', pattern: '**/*.{json,yaml,yml,json5}' }],
		new OptifyCodeActionProvider(),
		{
			providedCodeActionKinds: OptifyCodeActionProvider.providedCodeActionKinds
		}
	);

	const hoverProvider = vscode.languages.registerHoverProvider(
		[{ scheme: 'file', pattern: '**/*.{json,yaml,yml,json5}' }],
		dependentsHoverProvider
	);

	const onDidChangeDocument = vscode.workspace.onDidChangeTextDocument((event) => {
		if (isOptifyFeatureFile(event.document.fileName)) {
			diagnosticsProvider.updateDiagnostics(event.document);
		}
	});

	const onDidOpenDocument = vscode.workspace.onDidOpenTextDocument((document) => {
		if (isOptifyFeatureFile(document.fileName)) {
			diagnosticsProvider.updateDiagnostics(document);
			// Update dependents decoration when opening a document
			const editor = vscode.window.activeTextEditor;
			if (editor && editor.document === document) {
				dependentsProvider.updateDependentsDecoration(editor);
			}
		}
		updateOptifyFileContext();
	});

	const onDidChangeActiveEditor = vscode.window.onDidChangeActiveTextEditor((editor) => {
		updateOptifyFileContext();
		// Update dependents decoration when switching editors
		if (editor) {
			dependentsProvider.updateDependentsDecoration(editor);
		}
	});

	// Update dependents for the active editor on activation
	if (vscode.window.activeTextEditor) {
		dependentsProvider.updateDependentsDecoration(vscode.window.activeTextEditor);
	}

	context.subscriptions.push(
		outputChannel,
		previewCommand,
		linkProvider,
		definitionProvider,
		completionProvider,
		diagnosticCollection,
		codeActionProvider,
		hoverProvider,
		onDidChangeDocument,
		onDidOpenDocument,
		onDidChangeActiveEditor,
		dependentsProvider
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

		// outputChannel.appendLine(`Providing document links for ${document.fileName} | languageId: ${document.languageId}`);
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

export function deactivate() {
	console.debug("Deactivating Optify extension");
	for (const filePath of activePreviews.keys()) {
		cleanPreview(filePath);
	}
	// It should already be empty, but we'll clear it just in case.
	activePreviews.clear();
	clearProviderCache();
}
