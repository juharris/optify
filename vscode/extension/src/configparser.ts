import * as vscode from 'vscode';
import * as yaml from 'js-yaml';

export interface OptifyConfig {
	imports?: string[];
	options?: any;
}

/**
 * Utility functions for parsing configuration files
 */
export class ConfigParser {
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

	static findImportRange(text: string, importName: string, index: number, languageId: string): vscode.Range | undefined {
		switch (languageId) {
			case 'json':
				return ConfigParser.findImportRangeInJson(text, importName, index);
			case 'yaml':
				return ConfigParser.findImportRangeInYaml(text, importName, index);
			default:
				throw new Error(`Unsupported language ID: ${languageId}`);
		}
	}

	static parseImports(text: string, languageId: string): string[] | undefined {
		switch (languageId) {
			case 'json':
				return this.parseJson(text).imports;
			case 'yaml':
				return this.parseYaml(text).imports;
			default:
				throw new Error(`Unsupported language ID: ${languageId}`);
		}
	}

	private static parseJson(text: string): OptifyConfig {
		const config = JSON.parse(text);
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
		return config as OptifyConfig;
	}

	private static parseYaml(text: string): OptifyConfig {
		const config = yaml.load(text) as any;
		return this.validateConfig(config);
	}

	static findImportRangeInJson(text: string, importName: string, index: number): vscode.Range | undefined {
		try {
			const config = JSON.parse(text);
			if (!config.imports || !Array.isArray(config.imports)) {
				return undefined;
			}

			// Check if the import at the given index matches the importName
			if (index >= config.imports.length || config.imports[index] !== importName) {
				return undefined;
			}

			// Now find this specific occurrence in the text by position
			return this.findImportAtIndexInJsonText(text, index);
		} catch (error) {
			// If JSON parsing fails, fall back to regex approach
		}
		return undefined;
	}

	private static findImportAtIndexInJsonText(text: string, index: number): vscode.Range | undefined {
		const importsMatch = text.match(/"imports"\s*:\s*\[([^\]]*)\]/s);
		if (!importsMatch) {
			return undefined;
		}

		const importsContent = importsMatch[1];
		const importsArrayStartIndex = importsMatch.index! + importsMatch[0].indexOf(importsMatch[1]);

		// Find all string literals in the imports array
		const stringLiteralPattern = /"([^"]*)"/g;
		const matches: RegExpExecArray[] = [];
		let match;

		while ((match = stringLiteralPattern.exec(importsContent)) !== null) {
			matches.push(match);
		}

		if (index < matches.length) {
			const targetMatch = matches[index];
			const importName = targetMatch[1];
			// Calculate position relative to the start of the imports array content.
			// Add 1  to skip the opening quote.
			const startPos = importsArrayStartIndex + targetMatch.index! + 1;
			const endPos = startPos + importName.length;

			const startPosition = this.getPositionFromIndex(text, startPos);
			const endPosition = this.getPositionFromIndex(text, endPos);

			return new vscode.Range(startPosition, endPosition);
		}

		return undefined;
	}

	static findImportRangeInYaml(text: string, importName: string, index: number): vscode.Range | undefined {
		const lines = text.split('\n');
		let inImportsSection = false;
		let currentIndex = 0;
		let importLineIndex = -1;

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			const trimmedLine = line.trim();

			// Check if we're entering the imports section.
			if (trimmedLine.startsWith('imports:')) {
				inImportsSection = true;
				continue;
			}

			// If we're in imports section and hit another top-level key, exit.
			if (inImportsSection && trimmedLine && !trimmedLine.startsWith('-') && !trimmedLine.startsWith(' ') && trimmedLine.includes(':')) {
				break;
			}

			// Look for import items.
			if (inImportsSection && (trimmedLine.startsWith('-'))) {
				const importMatch = trimmedLine.match(/^-\s*["']?([^"'\n]+?)["']?\s*$/);
				if (importMatch) {
					const extractedImport = importMatch[1].trim();
					if (extractedImport === importName && currentIndex === index) {
						importLineIndex = i;
						break;
					}
					currentIndex++;
				}
			}
		}

		if (importLineIndex >= 0) {
			const line = lines[importLineIndex];
			const startCol = line.indexOf(importName);
			if (startCol >= 0) {
				const startPos = new vscode.Position(importLineIndex, startCol);
				const endPos = new vscode.Position(importLineIndex, startCol + importName.length);
				return new vscode.Range(startPos, endPos);
			}
		}

		return undefined;
	}

	static getPositionFromIndex(text: string, index: number): vscode.Position {
		const lines = text.substring(0, index).split('\n');
		return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
	}
}
