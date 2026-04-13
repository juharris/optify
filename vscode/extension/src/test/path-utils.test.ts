import * as assert from 'assert';
import * as path from 'path';
import * as vscode from 'vscode';
import { getCanonicalName, resolveFilePathArg } from '../path-utils';

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