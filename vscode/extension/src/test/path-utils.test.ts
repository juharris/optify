import * as assert from 'assert';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';
import * as vscode from 'vscode';
import { findOptifyRoot, getCanonicalName, getRelativeOptifyPath, isConfigFilePath, isOptifyFeatureFile, resolveFilePathArg } from '../path-utils';

suite('Utils Test Suite', () => {
	if (process.platform === 'win32') {
		test('getCanonicalName should handle Windows-style paths', () => {
			// Note: This test depends on the platform where it runs.
			// On Windows, path.join will use backslashes
			const optifyRoot = 'C:\\Users\\project\\configs';
			const filePath = 'C:\\Users\\project\\configs\\windows\\feature.json';
			const result = getCanonicalName(filePath, optifyRoot);
			assert.strictEqual(result, path.join('windows', 'feature'));
		});
	}
	else {
		// Non-Windows test cases
		test('getCanonicalName should return relative path without extension', () => {
			const optifyRoot = '/Users/project/configs';
			const filePath = '/Users/project/configs/feature.json';
			const result = getCanonicalName(filePath, optifyRoot);
			assert.strictEqual(result, 'feature');
		});

		test('getCanonicalName should handle nested directories', () => {
			const optifyRoot = '/Users/project/configs';
			const filePath = '/Users/project/configs/nested/deeply/feature.json';
			const result = getCanonicalName(filePath, optifyRoot);
			assert.strictEqual(result, path.join('nested', 'deeply', 'feature'));
		});

		test('getCanonicalName should handle files with dots in name', () => {
			const optifyRoot = '/Users/project/configs';
			const filePath = '/Users/project/configs/feature.v2.json';
			const result = getCanonicalName(filePath, optifyRoot);
			assert.strictEqual(result, 'feature.v2');
		});
	}
});

suite('resolveFilePathArg', () => {
	test('returns undefined for falsy values', () => {
		assert.strictEqual(resolveFilePathArg(undefined), undefined);
		assert.strictEqual(resolveFilePathArg(null), undefined);
		assert.strictEqual(resolveFilePathArg(''), undefined);
	});

	test('returns the string when given a string', () => {
		const filePath = '/some/path/feature.json';
		assert.strictEqual(resolveFilePathArg(filePath), filePath);
	});

	test('returns fsPath when given a Uri', () => {
		const uri = vscode.Uri.file('/some/path/feature.json');
		assert.strictEqual(resolveFilePathArg(uri), uri.fsPath);
	});

	test('returns undefined for unexpected types', () => {
		assert.strictEqual(resolveFilePathArg(42), undefined);
		assert.strictEqual(resolveFilePathArg({ other: 'prop' }), undefined);
	});
});

suite('getRelativeOptifyPath', () => {
	test('returns a relative path using forward slashes', () => {
		const result = getRelativeOptifyPath('/repo/configs/nested/file.json', '/repo');
		assert.strictEqual(result, 'configs/nested/file.json');
	});

	test('returns relative path unchanged on POSIX separators', () => {
		if (path.sep !== '/') {
			return;
		}

		const result = getRelativeOptifyPath('/repo/options/child/file.yaml', '/repo');
		assert.strictEqual(result, 'options/child/file.yaml');
	});

	test('returns undefined for files outside the optify root', () => {
		const result = getRelativeOptifyPath('/repo-other/configs/file.json', '/repo');
		assert.strictEqual(result, undefined);
	});
});

suite('isConfigFilePath', () => {
	const tempDirs: string[] = [];

	suiteTeardown(() => {
		for (const dir of tempDirs) {
			fs.rmSync(dir, { recursive: true, force: true });
		}
	});

	test('returns true for config-directory roots (no marker)', () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-config-root-'));
		tempDirs.push(root);
		const result = isConfigFilePath('any/subpath/file.yaml', root);
		assert.strictEqual(result, true);
	});

	test('returns true for marker roots when path starts with options-like top-level directory', () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-marker-root-'));
		tempDirs.push(root);
		fs.mkdirSync(path.join(root, '.optify'));
		const result = isConfigFilePath('configs/file.yaml', root);
		assert.strictEqual(result, true);
	});

	test('returns false for marker roots when path is outside options-like directories', () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-marker-root-'));
		tempDirs.push(root);
		fs.mkdirSync(path.join(root, '.optify'));
		const result = isConfigFilePath('src/file.yaml', root);
		assert.strictEqual(result, false);
	});

	test('returns true for marker roots when file is directly at root level', () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-marker-root-'));
		tempDirs.push(root);
		fs.mkdirSync(path.join(root, '.optify'));
		const result = isConfigFilePath('a.txt', root);
		assert.strictEqual(result, true);
	});

	test('returns true for config-directory roots with marker when path is nested', () => {
		const workspaceRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-marker-config-root-'));
		tempDirs.push(workspaceRoot);
		const configsRoot = path.join(workspaceRoot, 'configs');
		fs.mkdirSync(path.join(configsRoot, '.optify'), { recursive: true });
		const result = isConfigFilePath('templates/something.liquid', configsRoot);
		assert.strictEqual(result, true);
	});
});

suite('isOptifyFeatureFile', () => {
	const tempDirs: string[] = [];

	suiteTeardown(() => {
		for (const dir of tempDirs) {
			fs.rmSync(dir, { recursive: true, force: true });
		}
	});

	test('returns false for marker directory config file', () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-marker-config-'));
		tempDirs.push(root);
		fs.mkdirSync(path.join(root, '.optify'));
		const configPath = path.join(root, '.optify', 'config.json');
		fs.writeFileSync(configPath, '{}');

		assert.strictEqual(isOptifyFeatureFile(configPath, root), false);
	});

	test('returns false for nested marker directory files', () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-marker-nested-'));
		tempDirs.push(root);
		const nestedMarkerDir = path.join(root, '.optify', 'schemas');
		fs.mkdirSync(nestedMarkerDir, { recursive: true });
		const schemaPath = path.join(nestedMarkerDir, 'feature.json');
		fs.writeFileSync(schemaPath, '{}');

		assert.strictEqual(isOptifyFeatureFile(schemaPath, root), false);
	});

	test('returns true for JSON feature file under marker root', () => {
		const root = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-marker-feature-'));
		tempDirs.push(root);
		fs.mkdirSync(path.join(root, '.optify'));
		const featurePath = path.join(root, 'feature.json');
		fs.writeFileSync(featurePath, '{}');

		assert.strictEqual(isOptifyFeatureFile(featurePath, root), true);
	});
});

suite('findOptifyRoot', () => {
	const tempDirs: string[] = [];

	suiteTeardown(() => {
		for (const dir of tempDirs) {
			fs.rmSync(dir, { recursive: true, force: true });
		}
	});

	test('returns config directory when discovered by folder name', () => {
		const workspaceRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-workspace-'));
		tempDirs.push(workspaceRoot);
		const configsDir = path.join(workspaceRoot, 'configs');
		const nestedDir = path.join(configsDir, 'nested');
		fs.mkdirSync(nestedDir, { recursive: true });
		const filePath = path.join(nestedDir, 'feature.yaml');
		fs.writeFileSync(filePath, 'a: 1');

		const root = findOptifyRoot(filePath, workspaceRoot);
		assert.strictEqual(root, configsDir);
	});

	test('returns project root when discovered by .optify marker', () => {
		const workspaceRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-workspace-'));
		tempDirs.push(workspaceRoot);
		fs.mkdirSync(path.join(workspaceRoot, '.optify'));
		const nestedDir = path.join(workspaceRoot, 'src', 'nested');
		fs.mkdirSync(nestedDir, { recursive: true });
		const filePath = path.join(nestedDir, 'feature.yaml');
		fs.writeFileSync(filePath, 'a: 1');

		const root = findOptifyRoot(filePath, workspaceRoot);
		assert.strictEqual(root, workspaceRoot);
	});

	test('prefers nearest config directory over marker root above it', () => {
		const workspaceRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'optify-workspace-'));
		tempDirs.push(workspaceRoot);
		fs.mkdirSync(path.join(workspaceRoot, '.optify'));
		const configsDir = path.join(workspaceRoot, 'configs');
		const nestedDir = path.join(configsDir, 'nested');
		fs.mkdirSync(nestedDir, { recursive: true });
		const filePath = path.join(nestedDir, 'feature.yaml');
		fs.writeFileSync(filePath, 'a: 1');

		const root = findOptifyRoot(filePath, workspaceRoot);
		assert.strictEqual(root, configsDir);
	});
});
