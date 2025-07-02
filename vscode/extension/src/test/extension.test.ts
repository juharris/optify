import * as assert from 'assert';

import * as vscode from 'vscode';
import { buildOptifyPreview, findOptifyRoot } from '../extension';
import path from 'path';

suite('Extension Test Suite', () => {
	vscode.window.showInformationMessage('Start all tests.');

	test('buildOptifyPreview', () => {
		console.log('__dirname:', __dirname);
		const expectedRoot = path.join(__dirname, '../../src/test/configs');
		const featurePath = path.join(expectedRoot, 'feature.json');
		const optifyRoot = findOptifyRoot(featurePath, 'wtv');
		assert.equal(optifyRoot, expectedRoot);
		const preview = buildOptifyPreview('feature', expectedRoot);
		assert.ok(preview.startsWith('<!DOCTYPE html>'), preview);
		assert.ok(preview.includes('<div>Features: <code>["feature"]</code></div>\n\t<pre><code>{\n  "wtv": 3\n}</code></pre>'), preview);
	});
});
