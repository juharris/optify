import * as vscode from 'vscode';
import { findOptifyRoot, isOptifyFeatureFile } from './path-utils';
import { getOptionsProvider } from './providers';

/**
 * Provides completions for imports based on available features.
 */
export class OptifyCompletionProvider implements vscode.CompletionItemProvider {
	constructor(private outputChannel: vscode.OutputChannel) { }

	provideCompletionItems(
		document: vscode.TextDocument,
		position: vscode.Position,
		_token: vscode.CancellationToken,
		_context: vscode.CompletionContext
	): vscode.ProviderResult<vscode.CompletionItem[]> {
		const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
		if (!workspaceFolder) {
			return null;
		}

		const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);
		if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot)) {
			return null;
		}

		// Check if we're in an import context
		const lineText = document.lineAt(position).text;
		const linePrefix = lineText.substring(0, position.character);

		// Check for import patterns in JSON/YAML
		const isInImportContext =
			// JSON: "imports": ["<cursor>"]
			/["']imports["']\s*:\s*\[[^\]]*$/.test(linePrefix) ||
			// YAML: imports:\n  - <cursor>
			/^imports:\s*$/.test(linePrefix.trim()) ||
			/^\s*-\s*/.test(linePrefix) && document.getText().includes('imports:');

		if (!isInImportContext) {
			return null;
		}

		try {
			const provider = getOptionsProvider(optifyRoot);
			const features = provider.features();

			return features.map(feature => {
				const item = new vscode.CompletionItem(feature, vscode.CompletionItemKind.Module);
				item.detail = 'Optify feature';

				// Add quotes if needed
				const needsQuotes = !linePrefix.endsWith('"') && !linePrefix.endsWith("'");
				if (needsQuotes) {
					item.insertText = `"${feature}"`;
				}

				return item;
			});
		} catch (error) {
			this.outputChannel.appendLine(`Error getting completions: ${error}`);
			return null;
		}
	}
}