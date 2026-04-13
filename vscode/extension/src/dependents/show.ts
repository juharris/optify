import * as vscode from 'vscode';
import { getCanonicalName, findOptifyRoot, isOptifyFeatureFile } from '../path-utils';
import { getOptionsProvider } from '../providers';
import { getDecorationLineNumber } from './shared-utils';

/**
 * Show dependents at the top of the feature file as clickable inlay hints.
 * Each dependent name can be Cmd-clicked to navigate to that feature file.
 */
export class OptifyDependentsProvider implements vscode.InlayHintsProvider {
	private outputChannel: vscode.OutputChannel;
	private _onDidChangeInlayHints = new vscode.EventEmitter<void>();
	readonly onDidChangeInlayHints = this._onDidChangeInlayHints.event;

	constructor(outputChannel: vscode.OutputChannel) {
		this.outputChannel = outputChannel;
	}

	/** Trigger a refresh of the inlay hints (e.g. when options change). */
	public updateDependentsDecoration(_editor: vscode.TextEditor) {
		this._onDidChangeInlayHints.fire();
	}

	provideInlayHints(document: vscode.TextDocument, _range: vscode.Range): vscode.InlayHint[] | undefined {
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

			// We can't put the dependents on multiple lines as a decoration, so we'll just put them all on one line for now.
			const lineNum = getDecorationLineNumber(document);
			const lineRange = document.lineAt(lineNum).range;
			// Place the hint at the end of the line
			const position = lineRange.end;

			// Build label parts: prefix + dependent links separated by commas
			const labelParts: vscode.InlayHintLabelPart[] = [];

			const prefixPart = new vscode.InlayHintLabelPart(' dependents: [');
			labelParts.push(prefixPart);

			dependents.forEach((dep, index) => {
				const part = new vscode.InlayHintLabelPart(`"${dep}"`);
				const targetPath = featuresWithMetadata[dep]?.path();
				if (targetPath) {
					part.command = {
						title: dep,
						command: 'vscode.open',
						arguments: [vscode.Uri.file(targetPath)],
					};
				}
				labelParts.push(part);

				if (index < dependents.length - 1) {
					labelParts.push(new vscode.InlayHintLabelPart(', '));
				}
			});

			labelParts.push(new vscode.InlayHintLabelPart(' ],'));

			const hint = new vscode.InlayHint(position, labelParts);
			hint.paddingLeft = true;

			return [hint];
		} catch (error) {
			this.outputChannel.appendLine(`Error getting dependents: ${error}`);
			return undefined;
		}
	}

	public dispose() {
		this._onDidChangeInlayHints.dispose();
	}
}
