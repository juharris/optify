import * as vscode from 'vscode';
import { getCanonicalName, findOptifyRoot, isOptifyFeatureFile, resolveImportPath } from '../path-utils';
import { getOptionsProvider } from '../providers';

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

            // FIXME hover should only work on the decorations, but they are not real lines. Maybe this entire file has bad assumptions.
            // Provide hover only on lines where dependents are shown.
            // Handle title line, then each dependent, but not the closing bracket.
            // For JSON, we need to handle the `{` at the start of the file.
            // let offset = 0;
            // if (document.lineAt(0).text.trim().startsWith('{')) {
            //     offset = 1;
            // }
            // if (position.line > dependents.length + offset) {
            //     return undefined;
            // }
            // All dependents should be on the first line.
            if (position.line > 0) {
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
            const markdown = new vscode.MarkdownString(`## Dependents\nFeatures that depend on this one:\n${links.join('')}`);
            // Allow command URIs.
            markdown.isTrusted = true;

            return new vscode.Hover(markdown);
        } catch (error) {
            this.outputChannel.appendLine(`Error in hover provider: ${error}`);
            return undefined;
        }
    }
}