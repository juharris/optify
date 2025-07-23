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
		const isYamlFile = document.languageId === 'yaml' || document.languageId === 'yml';
		const isJsonFile = document.languageId === 'json' || document.languageId === 'jsonc' || document.languageId === 'json5';

		let isInImportContext = false;

		const documentText = document.getText();
		if (isJsonFile) {
			// JSON: "imports": ["<cursor>"] or 'imports': ['<cursor>']
			isInImportContext =
				/["']imports["']\s*:\s*\[[^\]]*$/.test(linePrefix) ||
				// Also check if we're inside an array after "imports"
				(documentText.includes('"imports"') || documentText.includes("'imports'")) &&
				/^\s*["']?[^"']*$/.test(linePrefix) &&
				this.isInsideImportsArray(document, position);
		} else if (isYamlFile) {
			// Don't show completions on comment lines
			if (lineText.trim().startsWith('#')) {
				return null;
			}

			// YAML: Check if we're in imports section
			const textBeforeCursor = documentText.substring(0, document.offsetAt(position));

			// Check if we're typing within quotes after a dash
			const isTypingQuotedValue = /^\s*-\s*("[^"]*|'[^']*)$/.test(linePrefix);

			// Check if we're after "imports:" and in a list item
			isInImportContext =
				// Match "imports:" followed by optional comments and newlines, then optional list item (including quotes)
				/imports:\s*(?:#[^\n]*)?(?:\n\s*(?:#[^\n]*)?)*\n\s*(?:-\s*(?:["']?[^#\n]*))?$/.test(textBeforeCursor) ||
				// Check if we're typing a quoted value in a list item under imports
				(isTypingQuotedValue && this.isInYamlImportsList(document, position)) ||
				// Already in a list under imports (handles more complex cases)
				this.isInYamlImportsList(document, position);
		}

		if (!isInImportContext) {
			return null;
		}

		try {
			const provider = getOptionsProvider(optifyRoot);
			const features = provider.features();

			return features.map(feature => {
				const item = new vscode.CompletionItem(feature, vscode.CompletionItemKind.Module);
				item.detail = 'Optify feature';

				// For JSON files, add quotes if needed
				if (isJsonFile) {
					const needsQuotes = !linePrefix.endsWith('"') && !linePrefix.endsWith("'");
					if (needsQuotes) {
						item.insertText = `"${feature}"`;
					}
				}
				// For YAML files, check if we're inside quotes
				else if (isYamlFile) {
					// Always insert just the feature name for YAML (no quotes)
					item.insertText = feature;
				}

				return item;
			});
		} catch (error) {
			this.outputChannel.appendLine(`Error getting completions: ${error}`);
			return null;
		}
	}

	private isInsideImportsArray(document: vscode.TextDocument, position: vscode.Position): boolean {
		const text = document.getText();
		const offset = document.offsetAt(position);

		// Find the last occurrence of "imports" before cursor
		const beforeCursor = text.substring(0, offset);
		const importsMatch = beforeCursor.match(/["']imports["']\s*:\s*\[/g);
		if (!importsMatch) {
			return false;
		}

		// Check if we're inside the array brackets
		const lastImportsIndex = beforeCursor.lastIndexOf(importsMatch[importsMatch.length - 1]);
		const afterImports = text.substring(lastImportsIndex);

		// Count brackets to see if we're inside the array
		let bracketCount = 0;
		for (let i = 0; i < afterImports.length && i < offset - lastImportsIndex; i++) {
			if (afterImports[i] === '[') { bracketCount++; }
			if (afterImports[i] === ']') { bracketCount--; }
		}

		return bracketCount > 0;
	}

	private isInYamlImportsList(document: vscode.TextDocument, position: vscode.Position): boolean {
		// Check if current line or previous lines indicate we're in an imports list
		let currentLine = position.line;

		// Check current line
		const currentLineText = document.lineAt(currentLine).text;
		const currentLineTrimmed = currentLineText.trim();

		// Skip comment lines when looking up
		const isListItem = /^\s*-\s*/.test(currentLineText);
		const isComment = currentLineTrimmed.startsWith('#');
		const startsWithQuote = currentLineTrimmed.startsWith('"') || currentLineTrimmed.startsWith("'");
		const isListItemWithQuote = /^\s*-\s*["']/.test(currentLineText);

		// Don't provide completions on comment lines
		if (isComment) {
			return false;
		}

		if (isListItem || currentLineTrimmed === '' || startsWithQuote || isListItemWithQuote) {
			// We're on a list item, comment, or empty line - check if it's under imports
			while (currentLine > 0) {
				currentLine--;
				const lineText = document.lineAt(currentLine).text;
				const lineTrimmed = lineText.trim();

				// Skip comment lines
				if (lineTrimmed.startsWith('#')) {
					continue;
				}

				// Found imports section (with or without comment)
				if (lineTrimmed === 'imports:' || lineTrimmed.match(/^imports:\s*(?:#.*)?$/)) {
					return true;
				}

				// Found another property at same or lower indentation level (not a list item)
				if (lineTrimmed && !lineTrimmed.startsWith('-') && /^\S/.test(lineText)) {
					// Check if this is actually a property (has a colon)
					if (lineTrimmed.includes(':')) {
						return false;
					}
				}
			}
		}

		// Check previous non-comment lines for imports:
		let checkLine = position.line;
		while (checkLine > 0 && checkLine > position.line - 5) { // Check up to 5 lines back
			checkLine--;
			const lineText = document.lineAt(checkLine).text.trim();

			// Skip comment lines
			if (lineText.startsWith('#')) {
				continue;
			}

			if (lineText === 'imports:' || lineText.match(/^imports:\s*(?:#.*)?$/)) {
				return true;
			}

			// Stop if we hit another property
			if (lineText && lineText.includes(':') && !lineText.startsWith('-')) {
				break;
			}
		}

		return false;
	}
}