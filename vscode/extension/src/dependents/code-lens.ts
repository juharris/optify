import * as vscode from 'vscode';
import { findOptifyRoot, getCanonicalName, getRelativeOptifyPath, isConfigFilePath, isOptifyFeatureFile } from '../path-utils';
import { getOptionsProvider } from '../providers';

/**
 * Shows dependency/reference context at the top of Optify files.
 *
 * Feature files show import dependents.
 * Non-feature files in options folders show features that reference that file.
 */
export class OptifyReferencesCodeLensProvider implements vscode.CodeLensProvider {
    private outputChannel: vscode.OutputChannel;
    private _onDidChangeCodeLenses = new vscode.EventEmitter<void>();
    readonly onDidChangeCodeLenses = this._onDidChangeCodeLenses.event;

    constructor(outputChannel: vscode.OutputChannel) {
        this.outputChannel = outputChannel;
    }

    public refresh(): void {
        this._onDidChangeCodeLenses.fire();
    }

    public provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] | undefined {
        const workspaceFolder = vscode.workspace.getWorkspaceFolder(document.uri);
        if (!workspaceFolder) {
            return undefined;
        }

        const optifyRoot = findOptifyRoot(document.fileName, workspaceFolder.uri.fsPath);
        if (!optifyRoot) {
            return undefined;
        }

        try {
            const provider = getOptionsProvider(optifyRoot);
            const featuresWithMetadata = provider.featuresWithMetadata();

            // Feature files: show the features that import this file.
            if (isOptifyFeatureFile(document.fileName, optifyRoot, workspaceFolder)) {
                this.outputChannel.appendLine(`[CodeLens] ${document.fileName} is a feature file`);
                const canonicalName = getCanonicalName(document.fileName, optifyRoot);
                const metadata = featuresWithMetadata[canonicalName];
                // FIXME Even when there are no dependents, we should still show the "Preview" lens.
                const dependents = metadata?.dependents();
                if (!dependents || dependents.length === 0) {
                    return undefined;
                }

                return this.createFeatureDependentsCodeLenses(document.uri, dependents, featuresWithMetadata);
            }

            // Non-feature config files: show which features reference this file.
            // Intentionally supports any file type the Rust backend can process (e.g. .txt).
            const relativePath = getRelativeOptifyPath(document.fileName, optifyRoot);
            if (!relativePath) {
                return undefined;
            }

            // Non-feature lenses intentionally only appear for options-scope files.
            // If the root comes from `<root>/.optify`, it is a project root and we
            // restrict to top-level `options|configs|configurations` directories.
            // If the root itself is an options/config directory, all files under that root are in scope.
            const isConfigPath = isConfigFilePath(relativePath, optifyRoot);
            this.outputChannel.appendLine(`[CodeLens] isConfigFilePath('${relativePath}', optifyRoot)=${isConfigPath}`);
            if (!isConfigPath) {
                this.outputChannel.appendLine(`[CodeLens] Not a config file path`);
                return undefined;
            }

            const referencingFeatures = provider.getFeaturesReferencingFile(relativePath);
            if (!referencingFeatures || referencingFeatures.length === 0) {
                this.outputChannel.appendLine(`[CodeLens] No referencing features`);
                return undefined;
            }

            return this.createFeatureReferencesCodeLenses(referencingFeatures, featuresWithMetadata);
        } catch (error) {
            this.outputChannel.appendLine(`Error creating Optify CodeLens references: ${error}`);
            return undefined;
        }
    }

    public dispose(): void {
        this._onDidChangeCodeLenses.dispose();
    }

    private createFeatureDependentsCodeLenses(
        documentUri: vscode.Uri,
        dependents: string[],
        featuresWithMetadata: Record<string, { path(): string | null }>
    ): vscode.CodeLens[] {
        // FIXME dependents should already be unique, but sorting is still fine.
        const sortedDependents = [...new Set(dependents)].sort((a, b) => a.localeCompare(b));
        const features = sortedDependents.flatMap(name => {
            const filePath = featuresWithMetadata[name]?.path();
            return filePath ? [{ name, path: filePath }] : [];
        });
        const range = new vscode.Range(0, 0, 0, 0);
        const lenses: vscode.CodeLens[] = [
            new vscode.CodeLens(range, {
                title: 'Preview',
                command: 'optify.previewFeature',
                arguments: [documentUri],
            }),
            new vscode.CodeLens(range, {
                title: 'Dependents',
                command: 'optify.openFeatureList',
                arguments: [features],
            }),
        ];

        for (const feature of features) {
            lenses.push(new vscode.CodeLens(range, {
                title: feature.name,
                command: 'vscode.open',
                arguments: [vscode.Uri.file(feature.path)],
            }));
        }

        return lenses;
    }

    private createFeatureReferencesCodeLenses(
        referencingFeatures: string[],
        featuresWithMetadata: Record<string, { path(): string | null }>
    ): vscode.CodeLens[] {
        const sortedFeatures = [...new Set(referencingFeatures)].sort((a, b) => a.localeCompare(b));
        // FIXME Don't allocate intermediate lists. Maybe we shouldn't use flatMap.
        const features = sortedFeatures.flatMap(name => {
            const filePath = featuresWithMetadata[name]?.path();
            return filePath ? [{ name, path: filePath }] : [];
        });
        const range = new vscode.Range(0, 0, 0, 0);
        const lenses: vscode.CodeLens[] = [
            new vscode.CodeLens(range, {
                title: 'Dependents',
                command: 'optify.openFeatureList',
                arguments: [features],
            }),
        ];

        for (const feature of features) {
            lenses.push(new vscode.CodeLens(range, {
                title: feature.name,
                command: 'vscode.open',
                arguments: [vscode.Uri.file(feature.path)],
            }));
        }

        return lenses;
    }
}
