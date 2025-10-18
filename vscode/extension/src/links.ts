import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { ConfigParser, ImportInfo, OptifyConfig } from './config-parser';
import { findOptifyRoot, isOptifyFeatureFile } from './path-utils';
import { getOptionsProvider } from './providers';
import { OptionsWatcher } from '@optify/config';

/**
 * Adds links to imports.
 */
export class OptifyDocumentLinkProvider implements vscode.DocumentLinkProvider {
    provideDocumentLinks(document: vscode.TextDocument): vscode.DocumentLink[] {
        const links: vscode.DocumentLink[] = [];
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) {
            return links;
        }

        const optifyRoot = findOptifyRoot(document.uri.fsPath, workspaceFolder.uri.fsPath);

        // Only provide links for Optify feature files
        if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot)) {
            return links;
        }

        const text = document.getText();
        const config = ConfigParser.parse(text, document.languageId);
        const importInfos = ConfigParser.findImportRanges(text, document.languageId, config);
        const provider = getOptionsProvider(optifyRoot);
        this.gatherImportLinks(provider, importInfos, links);
        this.gatherFileLinks(text, document, config, optifyRoot, links);

        return links;
    }

    private gatherFileLinks(text: string, document: vscode.TextDocument, config: OptifyConfig, optifyRoot: string, links: vscode.DocumentLink[]) {
        const fileRefs = ConfigParser.findFileReferences(text, document.languageId, config);
        for (const fileRef of fileRefs) {
            const fullPath = path.join(optifyRoot, fileRef.filePath);

            if (fs.existsSync(fullPath)) {
                const link = new vscode.DocumentLink(fileRef.range, vscode.Uri.file(fullPath));
                links.push(link);
            } else {
                // Create a command link to open quick open with the file path
                const commandUri = vscode.Uri.parse(`command:workbench.action.quickOpen?${encodeURIComponent(JSON.stringify(fileRef.filePath))}`);
                const link = new vscode.DocumentLink(fileRef.range, commandUri);
                links.push(link);
            }
        }
    }

    private gatherImportLinks(provider: OptionsWatcher, importInfos: ImportInfo[], links: vscode.DocumentLink[]): void {
        const featuresWithMetadata = provider.featuresWithMetadata();

        for (const importInfo of importInfos) {
            const targetPath = featuresWithMetadata[importInfo.name]?.path();
            if (targetPath) {
                const link = new vscode.DocumentLink(importInfo.range, vscode.Uri.file(targetPath));
                links.push(link);
            }
        }
    }
}