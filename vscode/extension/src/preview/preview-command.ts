import * as vscode from 'vscode';
import { outputChannel } from '../output-channel';
import { findOptifyRoot, getCanonicalName, isOptifyFeatureFile, resolveFilePathArg } from '../path-utils';
import { PreviewPanels } from './preview-panels';

/**
 * Registers the command to preview the built options for a feature file.
 */
export function registerPreviewCommand(previewPanels: PreviewPanels): vscode.Disposable {
	return vscode.commands.registerCommand('optify.previewFeature', async (filePathArg?: string | vscode.Uri) => {
		const filePath = resolveFilePathArg(filePathArg) ?? vscode.window.activeTextEditor?.document.fileName;
		if (!filePath) {
			vscode.window.showErrorMessage('No active editor found');
			return;
		}

		outputChannel.appendLine(`Opening preview for file: ${filePath}`);

		try {
			const uri = vscode.Uri.file(filePath);
			const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
			if (!workspaceFolder) {
				vscode.window.showErrorMessage("File must be in a workspace");
				return;
			}

			const optifyRoot = findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
			if (!optifyRoot) {
				vscode.window.showErrorMessage("Could not find Optify root directory");
				return;
			}

			if (!resolveFilePathArg(filePathArg) && !isOptifyFeatureFile(filePath, optifyRoot, workspaceFolder)) {
				vscode.window.showErrorMessage("Current file is not an Optify feature file");
				return;
			}

			const canonicalName = getCanonicalName(filePath, optifyRoot);
			await previewPanels.openOrReveal(filePath, optifyRoot, canonicalName);
		} catch (error) {
			const errorMessage = `Error opening Optify preview: ${error}`;
			console.error(errorMessage);
			outputChannel.appendLine(errorMessage);
			vscode.window.showErrorMessage(errorMessage);
		}
	});
}
