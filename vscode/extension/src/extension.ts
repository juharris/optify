import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';
import { OptifyCompletionProvider } from './completion';
import { ConfigParser } from './config-parser';
import { OptifyDefinitionProvider } from './definitions';
import { OptifyDependentsHoverProvider } from './dependents/hover';
import { OptifyDependentsProvider } from './dependents/show';
import { OptifyCodeActionProvider, OptifyDiagnosticsProvider } from './diagnostics';
import { OptifyDocumentLinkProvider } from './links';
import { findOptifyRoot, getCanonicalName, isOptifyFeatureFile, resolveFilePathArg } from './path-utils';
import { PreviewBuilder, PreviewWhileEditingOptions, PreviewData } from './preview';
import { clearProviderCache, getOptionsProvider, registerUpdateCallback } from './providers';

const outputChannel = vscode.window.createOutputChannel('Optify');

interface ActivePreview {
	panel: vscode.WebviewPanel
	documentChangeListener: vscode.Disposable
	debounceTimer?: NodeJS.Timeout
	updatePreview: () => void
	areConfigurableStringsEnabled: boolean
	customFeatures?: string[]
}

const activePreviews = new Map<string, ActivePreview>();

function readConfigurableStringsDefault(optifyRoot: string): boolean {
	try {
		const configPath = path.join(optifyRoot, '.optify', 'config.json');
		if (fs.existsSync(configPath)) {
			const configText = fs.readFileSync(configPath, 'utf8');
			const config = JSON.parse(configText);
			return config.areConfigurableStringsEnabled === true;
		}
	} catch (error) {
		// Ignore errors reading config
	}
	return false;
}

export function buildOptifyPreviewData(
	canonicalFeatures: string[],
	optifyRoot: string,
	editingOptions: PreviewWhileEditingOptions | undefined = undefined,
	areConfigurableStringsEnabled: boolean = false,
	configurableStringsDefault: boolean = false,
): PreviewData | { error: string } {
	const previewBuilder = new PreviewBuilder();
	try {
		// If some of the next lines fail in Rust from an unwrap or expect, then the exception is not caught.
		const provider = getOptionsProvider(optifyRoot);
		const data = previewBuilder.buildPreviewData(canonicalFeatures, provider, editingOptions, areConfigurableStringsEnabled, configurableStringsDefault);
		return data;
	} catch (error) {
		const message = `Failed to build preview${editingOptions ? " while editing" : ""}: ${error}`;
		console.error(message);
		if (!editingOptions) {
			vscode.window.showErrorMessage(message);
		}
		outputChannel.appendLine(message);
		const errorMessage = `${error}` + (editingOptions ? "\n\nIf the file was saved with any issues, then correct any issues and save the file to fix the preview." : "");
		return { error: errorMessage };
	}
}

function sendPreviewUpdate(panel: vscode.WebviewPanel, data: PreviewData | { error: string }) {
	panel.webview.postMessage({
		type: 'updateConfig',
		data: 'error' in data ? {
			features: [],
			config: {},
			dependents: null,
			isUnsaved: false,
			error: data.error,
			areConfigurableStringsEnabled: false,
			areConfigurableStringsEnabledDefault: false,
			allFeatureNames: [],
			featureAliases: {},
			featurePaths: {},
		} : data,
	});
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

	async function openOrRevealPreview(filePath: string, optifyRoot: string, canonicalName: string): Promise<void> {
		const existingPreview = activePreviews.get(filePath);
		if (existingPreview) {
			existingPreview.panel.reveal();
			return;
		}

		const configurableStringsDefault = readConfigurableStringsDefault(optifyRoot);

		const panel = vscode.window.createWebviewPanel(
			'optifyPreview',
			`Optify Preview: ${canonicalName}`,
			vscode.ViewColumn.Beside,
			{
				enableScripts: true,
				enableFindWidget: true,
				retainContextWhenHidden: true,
				localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'out')]
			}
		);

		const previewBuilder = new PreviewBuilder();
		panel.webview.html = previewBuilder.getPreviewHtmlShell(panel.webview, context.extensionUri);

		const updatePreview = () => {
			const preview = activePreviews.get(filePath);
			const data = buildOptifyPreviewData(
				preview?.customFeatures ?? [canonicalName],
				optifyRoot,
				undefined,
				preview?.areConfigurableStringsEnabled ?? configurableStringsDefault,
				configurableStringsDefault,
			);
			sendPreviewUpdate(panel, data);
		};

		// Handle messages from the webview
		panel.webview.onDidReceiveMessage(
			async (message) => {
				if (message.command === 'ready') {
					const preview = activePreviews.get(filePath);
					const initialData = buildOptifyPreviewData(
						preview?.customFeatures ?? [canonicalName],
						optifyRoot,
						undefined,
						preview?.areConfigurableStringsEnabled ?? configurableStringsDefault,
						configurableStringsDefault,
					);
					sendPreviewUpdate(panel, initialData);
				} else if (message.command === 'openFile') {
					if (message.path) {
						const uri = vscode.Uri.file(message.path);
						vscode.window.showTextDocument(uri);
					}
				} else if (message.command === 'setConfigurableStrings') {
					const preview = activePreviews.get(filePath);
					if (preview) {
						preview.areConfigurableStringsEnabled = message.enabled === true;
						const data = buildOptifyPreviewData(
							preview.customFeatures ?? [canonicalName],
							optifyRoot,
							undefined,
							preview.areConfigurableStringsEnabled,
							configurableStringsDefault,
						);
						sendPreviewUpdate(panel, data);
					}
				} else if (message.command === 'setFeatures') {
					const preview = activePreviews.get(filePath);
					if (preview) {
						const features: string[] = message.features ?? [];
						preview.customFeatures = features.length > 0 ? features : undefined;
						const targetFeatures = preview.customFeatures ?? [canonicalName];
						const data = buildOptifyPreviewData(
							targetFeatures,
							optifyRoot,
							undefined,
							preview.areConfigurableStringsEnabled,
							configurableStringsDefault,
						);
						sendPreviewUpdate(panel, data);
					}
				}
			},
			undefined,
			context.subscriptions
		);

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
					const documentText = event.document.getText();
					const config = ConfigParser.parse(documentText, event.document.languageId);
					const data = buildOptifyPreviewData(
						preview.customFeatures ?? [canonicalName],
						optifyRoot,
						{
							features: config.imports ?? [],
							overrides: config.options ? JSON.stringify(config.options) : undefined
						},
						preview.areConfigurableStringsEnabled,
						configurableStringsDefault,
					);
					sendPreviewUpdate(panel, data);
				}, EDIT_DEBOUNCE_MILLISECONDS);
			}
		});

		context.subscriptions.push(documentChangeListener);

		activePreviews.set(filePath, {
			panel,
			documentChangeListener,
			updatePreview,
			areConfigurableStringsEnabled: configurableStringsDefault,
		});

		// Clean up when panel is closed
		panel.onDidDispose(() => {
			cleanPreview(filePath);
		});
	}

	const previewCommand = vscode.commands.registerCommand('optify.previewFeature', async (filePathArg?: string | vscode.Uri) => {
		const filePath = resolveFilePathArg(filePathArg) ?? vscode.window.activeTextEditor?.document.fileName;
		if (!filePath) {
			vscode.window.showErrorMessage('No active editor found');
			return;
		}

		outputChannel.appendLine(`Opening preview for file: ${filePath}`);

		try {
			const uri = vscode.Uri.file(filePath);
			const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
			if (!workspaceFolder) {
				vscode.window.showErrorMessage("File must be in a workspace");
				return;
			}

			const optifyRoot = findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
			if (!optifyRoot) {
				vscode.window.showErrorMessage("Could not find Optify root directory");
				return;
			}

			if (!resolveFilePathArg(filePathArg) && !isOptifyFeatureFile(filePath, optifyRoot, workspaceFolder)) {
				vscode.window.showErrorMessage("Current file is not an Optify feature file");
				return;
			}

			const canonicalName = getCanonicalName(filePath, optifyRoot);
			await openOrRevealPreview(filePath, optifyRoot, canonicalName);
		} catch (error) {
			const errorMessage = `Error opening Optify preview: ${error}`;
			console.error(errorMessage);
			outputChannel.appendLine(errorMessage);
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

	const inlayHintsProvider = vscode.languages.registerInlayHintsProvider(
		[{ scheme: 'file', pattern: '**/*.{json,yaml,yml,json5}' }],
		dependentsProvider
	);

	const onDidChangeDocument = vscode.workspace.onDidChangeTextDocument((event) => {
		if (isOptifyFeatureFile(event.document.fileName)) {
			diagnosticsProvider.updateDiagnostics(event.document);
		}
	});

	const onDidOpenDocument = vscode.workspace.onDidOpenTextDocument((document) => {
		let filePath = document.fileName;
		switch (document.uri.scheme) {
			case 'git':
				filePath = filePath.replace(/\.git$/, '');
		}
		// console.debug(`onDidOpenDocument: filePath: ${filePath}`);
		const _isOptifyFeatureFile = isOptifyFeatureFile(filePath);
		if (_isOptifyFeatureFile) {
			diagnosticsProvider.updateDiagnostics(document);
			// Update dependents decoration when opening a document
			const editor = vscode.window.activeTextEditor;
			if (editor && editor.document === document) {
				dependentsProvider.updateDependentsDecoration(editor);
			}
		}
		updateOptifyFileContext(_isOptifyFeatureFile);
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
		inlayHintsProvider,
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

function updateOptifyFileContext(isOptifyFile?: boolean) {
	if (isOptifyFile === undefined) {
		const activeEditor = vscode.window.activeTextEditor;
		isOptifyFile = activeEditor ? isOptifyFeatureFile(activeEditor.document.fileName) : false;
	}
	vscode.commands.executeCommand('setContext', 'optify.isOptifyFile', isOptifyFile);
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
