import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';

export function findOptifyRoot(filePath: string, workspaceRoot: string): string | undefined {
	let currentDir = path.dirname(filePath);

	const configDirs = new Set(['options', 'configs', 'configurations']);
	const markerDirName = '.optify';
	while (currentDir !== path.dirname(currentDir)) {
		const currentDirName = path.basename(currentDir);
		if (configDirs.has(currentDirName)) {
			return currentDir;
		}

		const optifyConfigPath = path.join(currentDir, markerDirName);
		if (fs.existsSync(optifyConfigPath)) {
			return currentDir;
		}

		currentDir = path.dirname(currentDir);
		if (currentDir === workspaceRoot) {
			return undefined;
		}
	}

	return undefined;
}

export function isOptifyFeatureFile(filePath: string,
	optifyRoot: string | undefined = undefined,
	workspaceFolder: vscode.WorkspaceFolder | undefined = undefined): boolean {
	const ext = path.extname(filePath).toLowerCase();
	// We only support a few types of files in this extension and the config Rust crate only supports a few file types.
	if (!['.json', '.yaml', '.yml', '.json5'].includes(ext)) {
		return false;
	}

	// Check if file is in an Optify project by looking for root directory.
	if (!optifyRoot) {
		workspaceFolder ||= vscode.workspace.getWorkspaceFolder(vscode.Uri.file(filePath));
		if (!workspaceFolder) {
			return false;
		}

		optifyRoot = findOptifyRoot(filePath, workspaceFolder.uri.fsPath);
	}
	return optifyRoot !== undefined;
}

export function getCanonicalName(filePath: string, optifyRoot: string): string {
	const relativePath = path.relative(optifyRoot, filePath);
	const result = path.join(path.dirname(relativePath), path.basename(relativePath, path.extname(relativePath)));

	return result;
}

export function resolveImportPath(importName: string, optifyRoot: string): string | undefined {
	const extensions = ['.json', '.yaml', '.yml', '.json5'];

	// Try resolving relative to the optify root
	for (const ext of extensions) {
		const possiblePath = path.resolve(optifyRoot, importName + ext);
		if (fs.existsSync(possiblePath)) {
			return possiblePath;
		}
	}

	return undefined;
}