import * as vscode from 'vscode';
import { findOptifyRoot, getCanonicalName, isOptifyFeatureFile } from './path-utils';
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
		/*
		this.outputChannel.appendLine(`\n=== provideCompletionItems called for ${document.fileName} ===`);
		this.outputChannel.appendLine(`  Trigger kind: ${context.triggerKind} (0=Invoke, 1=TriggerCharacter, 2=TriggerForIncompleteCompletions)`);
		if (context.triggerCharacter) {
			this.outputChannel.appendLine(`  Trigger character: "${context.triggerCharacter}"`);
		}
		*/
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

			// Check if we're typing within quotes after a dash or just typed a quote
			const isTypingQuotedValue = /^\s*-\s*("[^"]*|'[^']*)$/.test(linePrefix);

			// Check if cursor is inside a quoted string (e.g., - "cur|sor")
			// This checks if we have an opening quote before cursor and closing quote after
			let isInsideQuotedString = false;
			const beforeCursorMatch = linePrefix.match(/^\s*-\s*(["'])/);
			if (beforeCursorMatch) {
				const quote = beforeCursorMatch[1];
				const afterCursor = lineText.substring(position.character);
				// Check if there's a closing quote after cursor
				if (afterCursor.includes(quote)) {
					// Make sure the quote in linePrefix is the opening quote
					const quotesInPrefix = (linePrefix.match(new RegExp(quote, 'g')) || []).length;
					if (quotesInPrefix === 1) {
						isInsideQuotedString = true;
					}
				}
			}

			// Debug logging
			/*
			this.outputChannel.appendLine(`YAML completion check at line ${position.line + 1}, char ${position.character}`);
			this.outputChannel.appendLine(`  lineText: "${lineText}"`);
			this.outputChannel.appendLine(`  linePrefix: "${linePrefix}"`);
			this.outputChannel.appendLine(`  isTypingQuotedValue: ${isTypingQuotedValue}`);
			this.outputChannel.appendLine(`  isInsideQuotedString: ${isInsideQuotedString}`);
			this.outputChannel.appendLine(`  textAfterCursor: "${lineText.substring(position.character)}"`);
			*/

			// Check if we're after "imports:" and in a list item
			const importRegexMatch = /imports:\s*(?:#[^\n]*)?(?:\n\s*(?:#[^\n]*)?)*\n\s*(?:-\s*(?:["']?[^#\n]*))?$/.test(textBeforeCursor);
			const isInYamlImportsList = this.isInYamlImportsList(document, position);

			isInImportContext =
				// Match "imports:" followed by optional comments and newlines, then optional list item (including quotes)
				importRegexMatch ||
				// Check if we're typing a quoted value in a list item under imports
				((isTypingQuotedValue || isInsideQuotedString) && isInYamlImportsList) ||
				// Already in a list under imports (handles more complex cases)
				isInYamlImportsList;

			/*
			this.outputChannel.appendLine(`  importRegexMatch: ${importRegexMatch}`);
			this.outputChannel.appendLine(`  isInYamlImportsList: ${isInYamlImportsList}`);
			this.outputChannel.appendLine(`  => isInImportContext: ${isInImportContext}`);
			*/
		}

		if (!isInImportContext) {
			this.outputChannel.appendLine('  => Returning null (not in import context)');
			return null;
		}

		try {
			const provider = getOptionsProvider(optifyRoot);
			const canonicalFeatureName = getCanonicalName(document.fileName, optifyRoot);
			const features = provider.features();
			const filteredFeatures = features.filter(feature =>
				canonicalFeatureName !== feature && !provider.hasConditions(feature));

			const currentLineTrimmed = lineText.trim();
			const needsSpaceAfterDash = /^\s*-$/.test(linePrefix);

			const completionItems = filteredFeatures.map(feature => {
				const item = new vscode.CompletionItem(feature, vscode.CompletionItemKind.Module);
				item.detail = 'Optify feature';

				// For JSON files, add quotes if needed
				if (isJsonFile) {
					const needsQuotes = !linePrefix.endsWith('"') && !linePrefix.endsWith("'");
					if (needsQuotes) {
						item.insertText = `"${feature}"`;
					}

					// Check for partial text that needs replacing
					const jsonPartialMatch = linePrefix.match(/(\w+)$/);
					if (jsonPartialMatch) {
						const partialText = jsonPartialMatch[1];
						const startCol = linePrefix.length - partialText.length;
						item.range = new vscode.Range(
							new vscode.Position(position.line, startCol),
							new vscode.Position(position.line, position.character)
						);
					}
				}
				// For YAML files, check if we need to add list item prefix
				else if (isYamlFile) {
					const isTypingQuotedValue = /^\s*-\s*("[^"]*|'[^']*)$/.test(linePrefix);
					// Check if we're inside quotes already
					if (isTypingQuotedValue) {
						// Don't add quotes, just the feature name
						item.insertText = feature;

						// Check if we have partial text inside quotes that needs replacing
						const quotedMatch = linePrefix.match(/^\s*-\s*(["'])(\w*)$/);
						if (quotedMatch) {
							const partialText = quotedMatch[2];
							if (partialText.length > 0) {
								const startCol = linePrefix.length - partialText.length;
								item.range = new vscode.Range(
									new vscode.Position(position.line, startCol),
									new vscode.Position(position.line, position.character)
								);
							}
						}
					} else {
						// Check if we're on an empty line after imports: or after a comment
						const needsListPrefix = currentLineTrimmed === '' ||
							(currentLineTrimmed.startsWith('#') && position.character === lineText.length);

						if (needsListPrefix) {
							// Calculate proper indentation
							let indent = '  '; // Default 2 spaces

							// Try to match indentation from previous list items
							const lines = document.getText().split('\n');
							for (let i = position.line - 1; i >= 0; i--) {
								const line = lines[i];
								const listMatch = line.match(/^(\s*)-\s*/);
								if (listMatch) {
									indent = listMatch[1];
									break;
								}
								// Stop if we hit imports: to avoid going too far
								if (line.trim() === 'imports:' || line.trim().startsWith('imports:')) {
									// Use 2 spaces after imports:
									indent = '  ';
									break;
								}
							}

							item.insertText = `${indent}- ${feature}`;
							// Replace the whole line if it's empty or just whitespace
							if (currentLineTrimmed === '') {
								const range = new vscode.Range(
									new vscode.Position(position.line, 0),
									new vscode.Position(position.line, lineText.length)
								);
								item.range = range;
							}
						} else {
							// Check if we need to add a space after dash
							if (needsSpaceAfterDash) {
								// Just a dash, add space before feature
								item.insertText = ` ${feature}`;
							} else {
								// Check if we're typing a partial word after "- "
								const afterDashMatch = linePrefix.match(/^\s*-\s+(\w*)$/);
								if (afterDashMatch) {
									// We have partial text after dash+space, need to replace it
									const partialText = afterDashMatch[1];
									const startCol = linePrefix.length - partialText.length;
									item.range = new vscode.Range(
										new vscode.Position(position.line, startCol),
										new vscode.Position(position.line, position.character)
									);
								}
								// Just insert the feature name
								item.insertText = feature;
							}
						}
					}
				}

				return item;
			});

			return completionItems;
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