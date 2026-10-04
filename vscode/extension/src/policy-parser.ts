import * as vscode from 'vscode';

export interface PolicyFeatureInfo {
	name: string
	range: vscode.Range
}

/**
 * Utility functions for parsing Optify policy files.
 */
export class PolicyParser {
	static findFeatureRanges(text: string, languageId: string): PolicyFeatureInfo[] {
		switch (languageId) {
			case 'json':
				return this.findFeatureRangesInJson(text);
			case 'yaml':
				return this.findFeatureRangesInYaml(text);
			default:
				return [];
		}
	}

	private static findFeatureRangesInJson(text: string): PolicyFeatureInfo[] {
		const results: PolicyFeatureInfo[] = [];
		const policyArrayPattern = /"(?:allow|block)"\s*:\s*\[([^\]]*)\]/g;
		let arrayMatch: RegExpExecArray | null;

		while ((arrayMatch = policyArrayPattern.exec(text)) !== null) {
			const arrayContent = arrayMatch[1];
			const stringsPattern = /"(?:\\.|[^"\\])*"/g;
			let stringMatch: RegExpExecArray | null;
			while ((stringMatch = stringsPattern.exec(arrayContent)) !== null) {
				try {
					const name = JSON.parse(stringMatch[0]) as string;
					const startIndex = arrayMatch.index + arrayMatch[0].indexOf(arrayContent) + stringMatch.index + 1;
					const startPosition = this.getPositionFromIndex(text, startIndex);
					const endPosition = this.getPositionFromIndex(text, startIndex + stringMatch[0].length - 2);
					results.push({ name, range: new vscode.Range(startPosition, endPosition) });
				} catch {
					// Ignore malformed string literals.
				}
			}
		}

		return results;
	}

	private static findFeatureRangesInYaml(text: string): PolicyFeatureInfo[] {
		const results: PolicyFeatureInfo[] = [];
		const lines = text.split('\n');

		for (let i = 0; i < lines.length; i++) {
			const policyListMatch = lines[i].match(/^(\s*)(?:allow|block)\s*:\s*(.*)$/);
			if (!policyListMatch) {
				continue;
			}

			const indent = policyListMatch[1].length;
			const inlineValue = policyListMatch[2].trim();
			if (inlineValue.startsWith('[') && inlineValue.endsWith(']')) {
				const listStart = lines[i].indexOf('[') + 1;
				const inlineItems = inlineValue.slice(1, -1);
				const itemPattern = /'([^']*)'|"((?:\\.|[^"\\])*)"|([^,\s][^,]*)/g;
				let itemMatch: RegExpExecArray | null;
				while ((itemMatch = itemPattern.exec(inlineItems)) !== null) {
					const name = itemMatch[1] ?? itemMatch[2] ?? itemMatch[3].trim();
					const rawValue = itemMatch[0];
					const startCol = listStart + itemMatch.index + (rawValue.startsWith('"') || rawValue.startsWith("'") ? 1 : 0);
					results.push({
						name,
						range: new vscode.Range(
							new vscode.Position(i, startCol),
							new vscode.Position(i, startCol + name.length)
						)
					});
				}
				continue;
			}

			if (inlineValue) {
				continue;
			}

			for (let listLine = i + 1; listLine < lines.length; listLine++) {
				const line = lines[listLine];
				const trimmedLine = line.trim();
				if (!trimmedLine || trimmedLine.startsWith('#')) {
					continue;
				}
				const listItem = line.match(/^(\s*)-\s*(.*?)\s*(?:#.*)?$/);
				if (!listItem || listItem[1].length <= indent) {
					if (line.length - line.trimStart().length <= indent) {
						break;
					}
					continue;
				}

				const rawValue = listItem[2].trim();
				const quote = rawValue[0] === '"' || rawValue[0] === "'" ? rawValue[0] : '';
				const name = quote && rawValue.endsWith(quote) ? rawValue.slice(1, -1) : rawValue;
				const startCol = line.indexOf(listItem[2]) + (quote ? 1 : 0);
				results.push({
					name,
					range: new vscode.Range(
						new vscode.Position(listLine, startCol),
						new vscode.Position(listLine, startCol + name.length)
					)
				});
			}
		}

		return results;
	}

	private static getPositionFromIndex(text: string, index: number): vscode.Position {
		const lines = text.substring(0, index).split('\n');
		return new vscode.Position(lines.length - 1, lines[lines.length - 1].length);
	}
}
