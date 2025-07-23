import * as assert from 'assert';
import * as path from 'path';
import { getCanonicalName } from '../path-utils';

suite('Utils Test Suite', () => {
	if (process.platform === 'win32') {
		test('getCanonicalName should handle Windows-style paths', () => {
			// Note: This test depends on the platform where it's run
			// On Windows, path.join will use backslashes
			const optifyRoot = 'C:\\Users\\project\\configs';
			const filePath = 'C:\\Users\\project\\configs\\windows\\feature.json';
			const result = getCanonicalName(filePath, optifyRoot);
			// The result will use the platform's path separator
			assert.strictEqual(result + process.platform, path.join('windows', 'feature'));
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