import * as vscode from 'vscode';
import { ConfigParser } from './config-parser';
import { findOptifyRoot, isOptifyFeatureFile, resolveImportPath } from './path-utils';

/**
 * Provides "Go to Definition" functionality for imports.
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
        if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot)) {
            return null;
        }

        const text = document.getText();
        const importInfos = ConfigParser.findImportRanges(text, document.languageId);

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
                const targetPath = resolveImportPath(importInfo.name, optifyRoot);
                if (targetPath) {
                    return new vscode.Location(vscode.Uri.file(targetPath), new vscode.Position(0, 0));
                }
            }
        }

        return null;
    }
}