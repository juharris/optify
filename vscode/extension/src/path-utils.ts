import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';

const CONFIG_DIRECTORIES = new Set(['options', 'configs', 'configurations']);
const MARKER_DIR_NAME = '.optify';

export function findOptifyRoot(filePath: string, workspaceRoot: string): string | undefined {
	let currentDir = path.dirname(filePath);
	const normalizedWorkspaceRoot = path.resolve(workspaceRoot);

	while (currentDir !== path.dirname(currentDir)) {
		const currentDirName = path.basename(currentDir);
		if (CONFIG_DIRECTORIES.has(currentDirName)) {
			return currentDir;
		}

		const optifyConfigPath = path.join(currentDir, MARKER_DIR_NAME);
		if (fs.existsSync(optifyConfigPath)) {
			return currentDir;
		}

		if (path.resolve(currentDir) === normalizedWorkspaceRoot) {
			return undefined;
		}

		currentDir = path.dirname(currentDir);
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

export function getRelativeOptifyPath(filePath: string, optifyRoot: string): string | undefined {
	const relativePath = path.relative(optifyRoot, filePath);
	if (!relativePath || relativePath.startsWith('..') || path.isAbsolute(relativePath)) {
		return undefined;
	}

	// The Rust provider expects forward slashes across platforms.
	if (path.sep === '/') {
		return relativePath;
	}
	return relativePath.split(path.sep).join('/');
}

/**
 * Returns true when a root-relative path should be treated as a config file path.
 *
 * This function exists because "inside the Optify root" can mean two different things:
 * 1) Root discovered by marker folder (`<root>/.optify`):
 *    - Root is a project folder.
 *    - Files directly at root level, or under top-level `options|configs|configurations`, are in scope.
 *    - Example: `a.txt` => true, `configs/feature.yaml` => true, `src/app.json` => false.
 * 2) Root discovered by config folder name (`options|configs|configurations`):
 *    - Root itself is already a config folder.
 *    - Everything under it is treated as config files.
 *    - Example: `nested/feature.yaml` => true.
 *
 * `relativePath` must already be relative to `optifyRoot` and use `/` separators
 * (for example from `getRelativeOptifyPath(...)`).
 */
export function isConfigFilePath(relativePath: string, optifyRoot: string): boolean {
	const rootDirName = path.basename(optifyRoot).toLowerCase();
	if (CONFIG_DIRECTORIES.has(rootDirName)) {
		// If the discovered root itself is a config directory, treat all files under
		// that root as config files, even when a marker directory is also present.
		return true;
	}

	const markerPath = path.join(optifyRoot, MARKER_DIR_NAME);
	if (!fs.existsSync(markerPath)) {
		// No marker folder means the root itself was found by config folder name,
		// so every file under that root is treated as a config file.
		return true;
	}

	// Marker folder means this is a project root.
	// Include files directly at the root level, or in top-level config directories.
	const parts = relativePath.split('/');
	if (parts.length === 1) {
		// File directly at root (e.g., `a.txt`)
		return true;
	}

	// File in subdirectory - only include if top-level is a config directory
	const topLevel = parts[0]?.toLowerCase();
	return CONFIG_DIRECTORIES.has(topLevel ?? '');
}

/**
 * Resolves a command argument to a file path string.
 * VS Code passes a Uri object from editor/title menus but a string from command URIs.
 */
export function resolveFilePathArg(arg: unknown): string | undefined {
	if (!arg) { return undefined; }
	if (typeof arg === 'string') { return arg; }
	if (typeof arg === 'object' && 'fsPath' in arg) { return (arg as vscode.Uri).fsPath; }
	return undefined;
}