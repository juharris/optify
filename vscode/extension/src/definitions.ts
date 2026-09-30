import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { ConfigParser } from './config-parser';
import { PolicyParser } from './policy-parser';
import { findOptifyRoot, isOptifyFeatureFile, isOptifyPoliciesFile } from './path-utils';
import { getOptionsProvider } from './providers';

/**
 * Provides "Go to Definition" functionality for imports and file paths.
 */
export class OptifyDefinitionProvider implements vscode.DefinitionProvider {
	provideDefinition(
		document: vscode.TextDocument,
		position: vscode.Position,
		_token: vscode.CancellationToken
	): vscode.ProviderResult<vscode.Definition> {
		const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
		if (!workspaceFolder) {
			return null;
		}

		const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);
		const isPoliciesFile = isOptifyPoliciesFile(document.fileName);
		if (!optifyRoot || (!isPoliciesFile && !isOptifyFeatureFile(document.fileName, optifyRoot))) {
			return null;
		}

		const provider = getOptionsProvider(optifyRoot);
		const featuresWithMetadata = provider.featuresWithMetadata();
		const text = document.getText();
		if (isPoliciesFile) {
			for (const featureInfo of PolicyParser.findFeatureRanges(text, document.languageId)) {
				const line = document.lineAt(featureInfo.range.start.line).text;
				const beforeChar = line.charAt(featureInfo.range.start.character - 1);
				const afterChar = line.charAt(featureInfo.range.end.character);
				const range = (beforeChar === '"' || beforeChar === "'") && afterChar === beforeChar
					? new vscode.Range(
						new vscode.Position(featureInfo.range.start.line, featureInfo.range.start.character - 1),
						new vscode.Position(featureInfo.range.end.line, featureInfo.range.end.character + 1)
					)
					: featureInfo.range;
				const targetPath = featuresWithMetadata[featureInfo.name]?.path();
				if (range.contains(position) && targetPath) {
					return new vscode.Location(vscode.Uri.file(targetPath), new vscode.Position(0, 0));
				}
			}
			return null;
		}

		const config = ConfigParser.parse(text, document.languageId);
		const importInfos = ConfigParser.findImportRanges(text, document.languageId, config);

		// Check if the cursor is on an import (including quotes)
		for (const importInfo of importInfos) {
			// Expand the range to include potential quotes around the import
			const line = document.lineAt(importInfo.range.start.line).text;
			const beforeChar = importInfo.range.start.character > 0 ? line.charAt(importInfo.range.start.character - 1) : '';
			const afterChar = importInfo.range.end.character < line.length ? line.charAt(importInfo.range.end.character) : '';

			let expandedRange = importInfo.range;
			if ((beforeChar === '"' || beforeChar === "'") && afterChar === beforeChar) {
				// Expand range to include quotes
				expandedRange = new vscode.Range(
					new vscode.Position(importInfo.range.start.line, importInfo.range.start.character - 1),
					new vscode.Position(importInfo.range.end.line, importInfo.range.end.character + 1)
				);
			}

			if (expandedRange.contains(position)) {
				const targetPath = featuresWithMetadata[importInfo.name]?.path();
				if (targetPath) {
					return new vscode.Location(vscode.Uri.file(targetPath), new vscode.Position(0, 0));
				}
			}
		}

		// Check if the cursor is on a file reference in options
		const fileRefs = ConfigParser.findFileReferences(text, document.languageId, config);
		for (const fileRef of fileRefs) {
			if (fileRef.range.contains(position)) {
				const fullPath = path.join(optifyRoot, fileRef.filePath);

				if (fs.existsSync(fullPath)) {
					return new vscode.Location(vscode.Uri.file(fullPath), new vscode.Position(0, 0));
				} else {
					// If file doesn't exist, trigger quick open with the file path
					vscode.commands.executeCommand('workbench.action.quickOpen', fileRef.filePath);
					return null;
				}
			}
		}

		return null;
	}
}