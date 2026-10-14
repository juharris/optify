import * as vscode from 'vscode';
import { findOptionKeySpan } from '../option-location';

/**
 * Opens a file and selects the key for the option at `keyPath`, or the closest ancestor key that is found.
 */
export async function openFile(filePath: string, keyPath?: string): Promise<void> {
	const document = await vscode.workspace.openTextDocument(vscode.Uri.file(filePath));
	const span = keyPath ? findOptionKeySpan(document.getText(), keyPath) : undefined;
	const selection = span && new vscode.Range(document.positionAt(span.start), document.positionAt(span.end));
	await vscode.window.showTextDocument(document, { selection });
}
