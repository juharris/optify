import * as vscode from 'vscode';
import { getCanonicalName, findOptifyRoot, isOptifyFeatureFile } from '../path-utils';
import { getOptionsProvider } from '../providers';
import { getDecorationLineNumber } from './shared-utils';

/**
 * Show dependents at the top of the feature file.
 */
export class OptifyDependentsProvider {
    private decorationType: vscode.TextEditorDecorationType;
    private outputChannel: vscode.OutputChannel;

    constructor(outputChannel: vscode.OutputChannel) {
        this.outputChannel = outputChannel;

        // Create a decoration type for grey text
        this.decorationType = vscode.window.createTextEditorDecorationType({
            before: {
                color: new vscode.ThemeColor('editorInlayHint.foreground'),
                fontStyle: 'italic',
                margin: '0 0 0 1rem',
            },
            rangeBehavior: vscode.DecorationRangeBehavior.ClosedOpen,
        });
    }

    public updateDependentsDecoration(editor: vscode.TextEditor) {
        if (!editor) {
            return;
        }

        const document = editor.document;
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) {
            editor.setDecorations(this.decorationType, []);
            return;
        }

        const optifyRoot = findOptifyRoot(document.fileName, workspaceFolder.uri.fsPath);
        if (!optifyRoot || !isOptifyFeatureFile(document.fileName, optifyRoot, workspaceFolder)) {
            editor.setDecorations(this.decorationType, []);
            return;
        }

        const canonicalName = getCanonicalName(document.fileName, optifyRoot);
        this.outputChannel.appendLine(`Checking dependents for "${canonicalName}"`);

        try {
            const provider = getOptionsProvider(optifyRoot);
            const featuresWithMetadata = provider.featuresWithMetadata();
            const metadata = featuresWithMetadata[canonicalName];
            const dependents = metadata?.dependents();

            if (!dependents || dependents.length === 0) {
                editor.setDecorations(this.decorationType, []);
                return;
            }

            const decorations: vscode.DecorationOptions[] = [];

            // We can't put the dependents on multiple lines as a decoration, so we'll just put them all on one line for now.
            const textComponents: string[] = ['"dependents": ['];
            dependents.forEach((dep, index) => {
                const isLast = index === dependents.length - 1;
                textComponents.push(` "${dep}"${isLast ? '' : ', '}`);
            });
            textComponents.push(' ],');

            const lineNum = getDecorationLineNumber(document);
            const lineRange = document.lineAt(lineNum).range;
            const contentText = textComponents.join('');
            const decoration: vscode.DecorationOptions = {
                range: new vscode.Range(lineNum, lineRange.end.character, lineNum + 1, lineRange.end.character),
                renderOptions: {
                    before: {
                        contentText: contentText
                    }
                }
            };

            decorations.push(decoration);
            editor.setDecorations(this.decorationType, decorations);

        } catch (error) {
            this.outputChannel.appendLine(`Error getting dependents: ${error}`);
            editor.setDecorations(this.decorationType, []);
        }
    }

    public dispose() {
        this.decorationType.dispose();
    }
}