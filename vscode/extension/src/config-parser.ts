import * as vscode from 'vscode';
import * as yaml from 'js-yaml';

export interface ImportInfo {
	name: string
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
	/**
	 * Find all import ranges in a document efficiently in a single parse
	 */
	static findImportRanges(text: string, languageId: string): ImportInfo[] {
		switch (languageId) {
			case 'json':
				return this.findImportRangesInJson(text);
			case 'yaml':
				return this.findImportRangesInYaml(text);
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

	static getPositionFromIndex(text: string, index: number): vscode.Position {
		const lines = text.substring(0, index).split('\n');
		return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
	}

	private static findImportRangesInJson(text: string): ImportInfo[] {
		const results: ImportInfo[] = [];

		try {
			const config = this.parseJson(text);
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

	private static findImportRangesInYaml(text: string): ImportInfo[] {
		const results: ImportInfo[] = [];
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
}
