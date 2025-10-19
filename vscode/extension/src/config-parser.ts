import * as vscode from 'vscode';
import * as yaml from 'js-yaml';

export interface ImportInfo {
	name: string
	range: vscode.Range
}

export interface FileReferenceInfo {
	filePath: string
	range: vscode.Range
}

export interface EqualsCondition {
	jsonPointer: string
	matches: string
}

export interface MatchesCondition {
	jsonPointer: string
	matches: string
}

export interface AndCondition {
	and: Condition[]
}

export interface OrCondition {
	or: Condition[]
}

export interface NotCondition {
	not: Condition
}

export type Condition = EqualsCondition | MatchesCondition | AndCondition | OrCondition | NotCondition;

export interface OptifyConfig {
	conditions?: Condition
	imports?: string[]
	options?: any
}

/**
 * Utility functions for parsing configuration files
 */
export class ConfigParser {
	static findConditionsRange(text: string, languageId: string, config?: OptifyConfig): vscode.Range | undefined {
		switch (languageId) {
			case 'json':
				return this.findConditionsRangeInJson(text, config);
			case 'yaml':
				return this.findConditionsRangeInYaml(text, config);
			default:
				return undefined;
		}
	}

	/**
	 * Find all import ranges in a document efficiently in a single parse
	 */
	static findImportRanges(text: string, languageId: string, config?: OptifyConfig): ImportInfo[] {
		switch (languageId) {
			case 'json':
				return this.findImportRangesInJson(text, config);
			case 'yaml':
				return this.findImportRangesInYaml(text, config);
			default:
				return [];
		}
	}

	/**
	 * Find all file references in the options section of a document.
	 */
	static findFileReferences(text: string, languageId: string, config?: OptifyConfig): FileReferenceInfo[] {
		switch (languageId) {
			case 'json':
				return this.findFileReferencesInJson(text, config);
			case 'yaml':
				return this.findFileReferencesInYaml(text, config);
			default:
				return [];
		}
	}

	static parse(text: string, languageId: string): OptifyConfig {
		switch (languageId) {
			case 'json':
				return this.parseJson(text);
			case 'yaml':
				return this.parseYaml(text);
			default:
				throw new Error(`Unsupported language ID: ${languageId}`);
		}
	}

	private static findConditionsRangeInJson(text: string, config?: OptifyConfig): vscode.Range | undefined {
		if (config && !config.conditions) {
			return undefined;
		}

		const conditionsMatch = text.match(/"conditions"\s*:/);
		if (!conditionsMatch) {
			return undefined;
		}

		const start = this.getPositionFromIndex(text, conditionsMatch.index!);
		const end = new vscode.Position(start.line, start.character + conditionsMatch[0].length);

		return new vscode.Range(start, end);
	}

	private static findConditionsRangeInYaml(text: string, config?: OptifyConfig): vscode.Range | undefined {
		if (config && !config.conditions) {
			return undefined;
		}

		const conditionsMatch = text.match(/^['"]?conditions['"]?\s*:\s*/m);
		if (!conditionsMatch) {
			return undefined;
		}

		const start = this.getPositionFromIndex(text, conditionsMatch.index!);
		const end = new vscode.Position(start.line, start.character + conditionsMatch[0].length);
		return new vscode.Range(start, end);
	}

	private static findFileReferencesInJson(text: string, config?: OptifyConfig): FileReferenceInfo[] {
		const results: FileReferenceInfo[] = [];

		try {
			config ||= this.parseJson(text);
			if (!config.options) {
				return results;
			}

			const optionsMatch = text.match(/"options"\s*:\s*\{/);
			if (!optionsMatch) {
				return results;
			}

			const optionsStartIndex = optionsMatch.index! + optionsMatch[0].length;

			const fileKeyPattern = /"file"\s*:\s*"([^"]*)"/g;
			let match;

			while ((match = fileKeyPattern.exec(text.substring(optionsStartIndex))) !== null) {
				const filePath = match[1];
				// Exclude the quotes from the range (only underline the file path)
				const startPos = optionsStartIndex + match.index! + match[0].indexOf('"' + filePath);
				// Include quotes
				const endPos = startPos + filePath.length + 2;

				const startPosition = this.getPositionFromIndex(text, startPos);
				const endPosition = this.getPositionFromIndex(text, endPos);

				results.push({
					filePath,
					range: new vscode.Range(startPosition, endPosition)
				});
			}
		} catch (error) {
			// Return empty array on parse error
		}

		return results;
	}

	private static findFileReferencesInYaml(text: string, config?: OptifyConfig): FileReferenceInfo[] {
		const results: FileReferenceInfo[] = [];

		try {
			config ||= this.parseYaml(text);
			if (!config.options) {
				return results;
			}
		} catch (error) {
			// Return empty array on parse error
			return results;
		}

		const lines = text.split('\n');
		let inOptionsSection = false;
		let optionsIndent = 0;

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			const trimmedLine = line.trim();

			// Ignore comments
			if (trimmedLine.startsWith('#')) {
				continue;
			}

			// Check if we're entering the options section
			if (trimmedLine.startsWith('options:')) {
				inOptionsSection = true;
				optionsIndent = line.indexOf('options:');
				continue;
			}

			// If we're in options section and hit another top-level key at same or lower indent, exit
			if (inOptionsSection && trimmedLine && line.indexOf(trimmedLine) <= optionsIndent && trimmedLine.includes(':') && !trimmedLine.startsWith('-')) {
				break;
			}

			if (inOptionsSection) {
				// Match various forms of file references:
				// 1. file: path/to/file.liquid
				// 2. file: "path/to/file.liquid"
				// 3. file: 'path/to/file.liquid'
				// 4. key: { file: "path/to/file.liquid" }
				// 5. key: { file: 'path/to/file.liquid' }
				// 6. key: { file: path/to/file.liquid }

				// Match standalone file key
				const standaloneMatch = trimmedLine.match(/^file\s*:\s*(["']?)([^"'\n]+?)\1\s*$/);
				if (standaloneMatch) {
					const quote = standaloneMatch[1];
					const filePath = standaloneMatch[2].trim();
					const startCol = line.indexOf(quote + filePath) + quote.length;
					const endCol = startCol + filePath.length;

					results.push({
						filePath,
						range: new vscode.Range(
							new vscode.Position(i, startCol),
							new vscode.Position(i, endCol)
						)
					});
					continue;
				}

				// Match inline object form: key: { file: "path" }
				const inlineMatch = trimmedLine.match(/\{\s*file\s*:\s*(["']?)([^"'\}]+?)\1\s*\}/);
				if (inlineMatch) {
					const quote = inlineMatch[1];
					const filePath = inlineMatch[2].trim();
					const fileKeyStart = line.indexOf('file');
					const pathStart = line.indexOf(quote + filePath, fileKeyStart) + quote.length;
					const pathEnd = pathStart + filePath.length;

					results.push({
						filePath,
						range: new vscode.Range(
							new vscode.Position(i, pathStart),
							new vscode.Position(i, pathEnd)
						)
					});
				}
			}
		}

		return results;
	}

	private static findImportRangesInJson(text: string, config?: OptifyConfig): ImportInfo[] {
		const results: ImportInfo[] = [];

		try {
			config ||= this.parseJson(text);
			if (!config.imports) {
				return results;
			}

			// Find the imports array in the text
			const importsMatch = text.match(/"imports"\s*:\s*\[([^\]]*)\]/s);
			if (!importsMatch) {
				return results;
			}

			const importsContent = importsMatch[1];
			const importsArrayStartIndex = importsMatch.index! + importsMatch[0].indexOf(importsMatch[1]);

			// Find all string literals in the imports array
			const stringLiteralPattern = /"([^"]*)"/g;
			let match;
			let index = 0;

			while ((match = stringLiteralPattern.exec(importsContent)) !== null) {
				if (index < config.imports.length) {
					const importName = match[1];
					// Exclude the quotes from the range (only underline the import name)
					const startPos = importsArrayStartIndex + match.index! + 1;
					const endPos = startPos + importName.length;

					const startPosition = this.getPositionFromIndex(text, startPos);
					const endPosition = this.getPositionFromIndex(text, endPos);

					results.push({
						name: importName,
						range: new vscode.Range(startPosition, endPosition)
					});
				}
				index++;
			}
		} catch (error) {
			// Return empty array on parse error
		}

		return results;
	}

	private static findImportRangesInYaml(text: string, config?: OptifyConfig): ImportInfo[] {
		const results: ImportInfo[] = [];
		if (config && !config.imports) {
			return results;
		}

		const lines = text.split('\n');
		let inImportsSection = false;

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			const trimmedLine = line.trim();

			// Check if we're entering the imports section.
			if (trimmedLine.startsWith('imports:')) {
				inImportsSection = true;
				continue;
			}

			// Ignore comments.
			if (trimmedLine.startsWith('#')) {
				continue;
			}

			// If we're in imports section and hit another top-level key, exit.
			if (inImportsSection && trimmedLine && !trimmedLine.startsWith('-') && trimmedLine.includes(':')) {
				break;
			}

			// Look for import items
			if (inImportsSection && trimmedLine.startsWith('-')) {
				// Match the import with optional quotes, capturing both the full match and just the import name
				const importMatch = trimmedLine.match(/^-\s*(["']?)([^"'\n]+?)\1\s*$/);
				if (importMatch) {
					const quote = importMatch[1];  // Empty string if no quotes, otherwise " or '
					const importName = importMatch[2].trim();

					// Find where the import name starts (excluding quote)
					let startCol = line.indexOf(quote + importName) + quote.length;
					if (startCol >= quote.length) {
						// Range covers only the import name, not the quotes
						const endCol = startCol + importName.length;

						results.push({
							name: importName,
							range: new vscode.Range(
								new vscode.Position(i, startCol),
								new vscode.Position(i, endCol)
							)
						});
					}
				}
			}
		}

		return results;
	}

	private static getPositionFromIndex(text: string, index: number): vscode.Position {
		const lines = text.substring(0, index).split('\n');
		return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
	}

	private static parseJson(text: string): OptifyConfig {
		const config = JSON.parse(text);
		return this.validateConfig(config);
	}

	private static parseYaml(text: string): OptifyConfig {
		const config = yaml.load(text) as any;
		return this.validateConfig(config);
	}

	private static validateConfig(config: any): OptifyConfig {
		if (!config || typeof config !== 'object') {
			throw new Error(`Text must be a valid JSON object. Got: ${JSON.stringify(config)}`);
		}
		if (Array.isArray(config.imports) && !config.imports.every((imp: any) => typeof imp === 'string')) {
			throw new Error(`Expected "imports" to be an array of strings. Got: "${JSON.stringify(config.imports)}"`);
		}

		if (config.options && typeof config.options !== 'object') {
			throw new Error(`Expected "options" to be an object. Got: "${JSON.stringify(config.options)}"`);
		}

		if (config.conditions && typeof config.conditions !== 'object') {
			throw new Error(`Expected "conditions" to be an object. Got: "${JSON.stringify(config.conditions)}"`);
		}
		return config as OptifyConfig;
	}
}
