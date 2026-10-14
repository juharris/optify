import * as vscode from 'vscode';
import { COMPLETION_TRIGGER_CHARACTERS, OptifyCompletionProvider } from './completion';
import { OptifyDefinitionProvider } from './definitions';
import { OptifyReferencesCodeLensProvider } from './dependents/code-lens';
import { OptifyCodeActionProvider, OptifyDiagnosticsProvider } from './diagnostics';
import { OptifyDocumentLinkProvider } from './links';
import { outputChannel } from './output-channel';
import { findOptifyRoot, isOptifyFeatureFile, isOptifyPoliciesFile } from './path-utils';
import { registerPreviewCommand } from './preview/preview-command';
import { PreviewPanels } from './preview/preview-panels';
import { clearProviderCache, registerUpdateCallback } from './providers';

let previewPanels: PreviewPanels | undefined;

export function activate(context: vscode.ExtensionContext) {
	outputChannel.appendLine('Optify extension is now active!');

	// Set up context for when clauses
	updateOptifyFileContext();

	const referencesCodeLensProvider = new OptifyReferencesCodeLensProvider(outputChannel);
	const panels = new PreviewPanels(context);
	previewPanels = panels;

	// Register callback to update previews and dependents when options change
	registerUpdateCallback(() => {
		panels.updateAll();
		referencesCodeLensProvider.refresh();
	});

	const openFeatureListCommand = vscode.commands.registerCommand(
		'optify.openFeatureList',
		async (features: { name: string; path: string }[]) => {
			const items = features.map(f => ({ label: f.name, filePath: f.path }));
			const selected = await vscode.window.showQuickPick(items, {
				placeHolder: 'Select a feature to open',
			});
			if (selected) {
				await vscode.commands.executeCommand('vscode.open', vscode.Uri.file(selected.filePath));
			}
		},
	);

	const previewCommand = registerPreviewCommand(panels);

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

	const codeLensProvider = vscode.languages.registerCodeLensProvider(
		[{ scheme: 'file', pattern: '**/*.{json,json5,md,liquid,txt,yaml,yml}' }],
		referencesCodeLensProvider
	);

	const onDidChangeDocument = vscode.workspace.onDidChangeTextDocument((event) => {
		const workspaceFolder = vscode.workspace.getWorkspaceFolder(event.document.uri);
		const optifyRoot = workspaceFolder
			? findOptifyRoot(event.document.fileName, workspaceFolder.uri.fsPath)
			: undefined;
		if (isOptifyFeatureFile(event.document.fileName, optifyRoot) ||
			(optifyRoot && isOptifyPoliciesFile(event.document.fileName, optifyRoot))) {
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
		const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
		const optifyRoot = workspaceFolder
			? findOptifyRoot(filePath, workspaceFolder.uri.fsPath)
			: undefined;
		const _isOptifyFeatureFile = isOptifyFeatureFile(filePath, optifyRoot);
		if (_isOptifyFeatureFile || (optifyRoot && isOptifyPoliciesFile(filePath, optifyRoot))) {
			diagnosticsProvider.updateDiagnostics(document);
		}
		referencesCodeLensProvider.refresh();
		updateOptifyFileContext(_isOptifyFeatureFile);
	});

	const onDidChangeActiveEditor = vscode.window.onDidChangeActiveTextEditor((editor) => {
		updateOptifyFileContext();
		if (editor) {
			referencesCodeLensProvider.refresh();
		}
	});

	// Refresh CodeLens for the active editor on activation
	if (vscode.window.activeTextEditor) {
		referencesCodeLensProvider.refresh();
	}

	context.subscriptions.push(
		outputChannel,
		openFeatureListCommand,
		previewCommand,
		linkProvider,
		definitionProvider,
		completionProvider,
		diagnosticCollection,
		codeActionProvider,
		codeLensProvider,
		onDidChangeDocument,
		onDidOpenDocument,
		onDidChangeActiveEditor,
		referencesCodeLensProvider
	);
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
	previewPanels?.dispose();
	previewPanels = undefined;
	clearProviderCache();
}
