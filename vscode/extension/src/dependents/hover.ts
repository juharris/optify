import * as vscode from 'vscode';
import { findOptifyRoot, getCanonicalName, isOptifyFeatureFile, resolveImportPath } from '../path-utils';
import { getOptionsProvider } from '../providers';
import { getDecorationLineNumber } from './shared-utils';

/**
 * Provides hover information for Optify dependents with links to feature files.
 */
export class OptifyDependentsHoverProvider implements vscode.HoverProvider {
    private outputChannel: vscode.OutputChannel;

    constructor(outputChannel: vscode.OutputChannel) {
        this.outputChannel = outputChannel;
    }

    provideHover(document: vscode.TextDocument, position: vscode.Position): vscode.Hover | undefined {
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) {
            return undefined;
        }

        const optifyRoot = findOptifyRoot(document.fileName, workspaceFolder.uri.fsPath);
        if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot, workspaceFolder)) {
            return undefined;
        }

        const canonicalName = getCanonicalName(document.fileName, optifyRoot);

        try {
            const provider = getOptionsProvider(optifyRoot);
            const featuresWithMetadata = provider.featuresWithMetadata();
            const metadata = featuresWithMetadata[canonicalName];
            const dependents = metadata?.dependents();

            if (!dependents || dependents.length === 0) {
                return undefined;
            }

            const lineNumber = getDecorationLineNumber(document);
            if (position.line !== lineNumber) {
                return undefined;
            }

            // Create markdown with links to dependent files
            const links = dependents.map(dep => {
                const targetPath = resolveImportPath(dep, optifyRoot);
                if (targetPath) {
                    const uri = vscode.Uri.file(targetPath);
                    return `* [${dep}](${uri.toString()})\n`;
                }
                return dep;
            });
            const markdown = new vscode.MarkdownString(`## Dependents\nFeatures that import this one:\n${links.join('')}`);
            // Allow command URIs.
            markdown.isTrusted = true;

            return new vscode.Hover(markdown);
        } catch (error) {
            this.outputChannel.appendLine(`Error in hover provider: ${error}`);
            return undefined;
        }
    }
}