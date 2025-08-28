import * as vscode from 'vscode';
import { ConfigParser } from './config-parser';
import { findOptifyRoot, getCanonicalName, isOptifyFeatureFile } from './path-utils';
import { getOptionsProvider } from './providers';
import { OptionsWatcher } from '@optify/config';

/**
 * Validates files such as validating imports.
*/
export class OptifyDiagnosticsProvider {
	outputChannel = vscode.window.createOutputChannel('Optify');
	constructor(private diagnosticCollection: vscode.DiagnosticCollection) { }

	updateDiagnostics(document: vscode.TextDocument): void {
		// this.outputChannel.appendLine(`Updating diagnostics for ${document.fileName} | languageId: ${document.languageId}`);
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

		try {
			const provider = getOptionsProvider(optifyRoot);

			const currentFileCanonicalName = getCanonicalName(document.uri.fsPath, optifyRoot);
			this.checkImports(text, document, provider, diagnostics, currentFileCanonicalName);

			this.diagnosticCollection.set(document.uri, diagnostics);
		} catch (error) {
			console.error(`Error getting diagnostics and suggestions for ${optifyRoot}:`, error);
			this.outputChannel.appendLine(`Error getting diagnostics and suggestions for ${optifyRoot}: ${error}`);
			return;
		}
	}

	private checkImports(
		text: string,
		document: vscode.TextDocument,
		provider: OptionsWatcher,
		diagnostics: vscode.Diagnostic[],
		currentFileCanonicalName: string
	) {
		const importInfos = ConfigParser.findImportRanges(text, document.languageId);
		const featuresWithMetadata = provider.featuresWithMetadata();
		for (const importInfo of importInfos) {
			const targetPath = featuresWithMetadata[importInfo.name]?.path();
			if (!targetPath) {
				// Check if it might be a feature alias
				try {
					const canonicalName = provider.getCanonicalFeatureName(importInfo.name);
					if (canonicalName && canonicalName !== importInfo.name) {
						// It's an alias - create diagnostic with code for quick fix
						const diagnostic = new vscode.Diagnostic(
							importInfo.range,
							`Use '${canonicalName}' for clarity and to help navigate to the file. '${importInfo.name}' is an alias.`,
							vscode.DiagnosticSeverity.Error
						);
						// Store alias info in the code object
						diagnostic.code = {
							value: `feature-alias:${importInfo.name}:${canonicalName}`,
							target: vscode.Uri.parse('https://github.com/juharris/optify')
						};
						diagnostics.push(diagnostic);
					} else {
						// Not an alias, just unresolved
						const diagnostic = new vscode.Diagnostic(
							importInfo.range,
							`Cannot resolve import '${importInfo.name}'`,
							vscode.DiagnosticSeverity.Error
						);
						diagnostics.push(diagnostic);
					}
				} catch {
					// If getCanonicalFeatureName fails, treat as unresolved
					const diagnostic = new vscode.Diagnostic(
						importInfo.range,
						`Cannot resolve import '${importInfo.name}'`,
						vscode.DiagnosticSeverity.Error
					);
					diagnostics.push(diagnostic);
				}
			} else if (importInfo.name === currentFileCanonicalName) {
				const diagnostic = new vscode.Diagnostic(
					importInfo.range,
					"A file cannot import itself",
					vscode.DiagnosticSeverity.Error
				);
				diagnostics.push(diagnostic);
			}
		}
	}
}

/**
 * Provides quick fixes for feature alias suggestions.
 */
export class OptifyCodeActionProvider implements vscode.CodeActionProvider {
	public static readonly providedCodeActionKinds = [
		vscode.CodeActionKind.QuickFix,
		vscode.CodeActionKind.SourceFixAll
	];

	provideCodeActions(
		document: vscode.TextDocument,
		_range: vscode.Range | vscode.Selection,
		context: vscode.CodeActionContext,
		_token: vscode.CancellationToken
	): vscode.CodeAction[] {
		const actions: vscode.CodeAction[] = [];

		// Check if this is a source.fixAll request
		if (context.only && context.only.contains(vscode.CodeActionKind.SourceFixAll)) {
			const fixAllAction = this.createFixAllAliasesAction(document);
			if (fixAllAction) {
				actions.push(fixAllAction);
			}
			return actions;
		}

		// Otherwise, provide quick fixes for individual diagnostics
		for (const diagnostic of context.diagnostics) {
			// Check if this is our feature alias diagnostic
			if (diagnostic.code &&
				typeof diagnostic.code === 'object' &&
				'value' in diagnostic.code &&
				typeof diagnostic.code.value === 'string' &&
				diagnostic.code.value.startsWith('feature-alias:')) {

				const action = this.createReplaceAliasAction(document, diagnostic);
				if (action) {
					actions.push(action);
				}
			}
		}

		return actions;
	}

	private createReplaceAliasAction(document: vscode.TextDocument, diagnostic: vscode.Diagnostic): vscode.CodeAction | undefined {
		// Extract alias and canonical name from the code value
		if (!diagnostic.code || typeof diagnostic.code !== 'object' || !('value' in diagnostic.code) || typeof diagnostic.code.value !== 'string') {
			return undefined;
		}

		const parts = diagnostic.code.value.split(':');
		if (parts.length !== 3) {
			return undefined;
		}

		const [, alias, canonical] = parts;

		const action = new vscode.CodeAction(
			`Replace with '${canonical}'`,
			vscode.CodeActionKind.QuickFix
		);

		action.edit = new vscode.WorkspaceEdit();

		// Get the actual text at the range to preserve quotes
		const lineText = document.lineAt(diagnostic.range.start.line).text;
		const rangeText = lineText.substring(diagnostic.range.start.character, diagnostic.range.end.character);

		// Replace just the alias part, preserving any quotes
		const newText = rangeText.replace(alias, canonical);
		action.edit.replace(document.uri, diagnostic.range, newText);

		action.diagnostics = [diagnostic];
		action.isPreferred = true;

		return action;
	}

	private createFixAllAliasesAction(document: vscode.TextDocument): vscode.CodeAction | undefined {
		const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
		if (!workspaceFolder) {
			return undefined;
		}

		const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);
		if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot)) {
			return undefined;
		}

		const action = new vscode.CodeAction(
			'Convert all feature aliases to canonical names',
			vscode.CodeActionKind.SourceFixAll
		);

		action.edit = new vscode.WorkspaceEdit();

		try {
			const provider = getOptionsProvider(optifyRoot);
			const text = document.getText();
			const importInfos = ConfigParser.findImportRanges(text, document.languageId);

			let hasAliases = false;
			for (const importInfo of importInfos) {
				try {
					const canonicalName = provider.getCanonicalFeatureName(importInfo.name);
					if (canonicalName && canonicalName !== importInfo.name) {
						hasAliases = true;
						// Create an edit to replace the alias with canonical name
						const lineText = document.lineAt(importInfo.range.start.line).text;
						const rangeText = lineText.substring(importInfo.range.start.character, importInfo.range.end.character);
						const newText = rangeText.replace(importInfo.name, canonicalName);

						action.edit.replace(document.uri, importInfo.range, newText);
					}
				} catch {
					// If getCanonicalFeatureName fails, skip this import
				}
			}

			return hasAliases ? action : undefined;
		} catch {
			return undefined;
		}
	}
}
