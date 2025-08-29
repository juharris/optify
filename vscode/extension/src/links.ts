import * as vscode from 'vscode';
import { ConfigParser } from './config-parser';
import { findOptifyRoot, isOptifyFeatureFile } from './path-utils';
import { getOptionsProvider } from './providers';

/**
 * Adds links to imports.
 */
export class OptifyDocumentLinkProvider implements vscode.DocumentLinkProvider {
    provideDocumentLinks(document: vscode.TextDocument): vscode.DocumentLink[] {
        const links: vscode.DocumentLink[] = [];
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) {
            return links; // No workspace folder, no links
        }

        const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);

        // Only provide links for Optify feature files
        if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot)) {
            return links;
        }

        // outputChannel.appendLine(`Providing document links for ${document.fileName} | languageId: ${document.languageId}`);
        const text = document.getText();
        const importInfos = ConfigParser.findImportRanges(text, document.languageId);
        const provider = getOptionsProvider(optifyRoot);
        const featuresWithMetadata = provider.featuresWithMetadata();

        for (const importInfo of importInfos) {
            const targetPath = featuresWithMetadata[importInfo.name]?.path();
            if (targetPath) {
                const link = new vscode.DocumentLink(importInfo.range, vscode.Uri.file(targetPath));
                links.push(link);
            }
        }

        return links;
    }
}