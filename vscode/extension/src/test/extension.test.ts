import * as assert from 'assert';

import * as vscode from 'vscode';
import { buildOptifyPreview } from '../extension';
import { findOptifyRoot } from '../path-utils';
import path from 'path';

suite('Extension Test Suite', () => {
	vscode.window.showInformationMessage('Start all tests.');

	test('buildOptifyPreview', () => {
		const expectedRoot = path.join(__dirname, '../../src/test/configs');
		const featurePath = path.join(expectedRoot, 'feature.json');
		const optifyRoot = findOptifyRoot(featurePath, 'wtv');
		assert.equal(optifyRoot, expectedRoot);
		const preview = buildOptifyPreview(['feature'], expectedRoot);
		assert.ok(preview.startsWith('<!DOCTYPE html>'), preview);
		assert.ok(preview.includes('<div>Features:<pre><code>[<span class="string">"feature"</span>]</code></pre></div>\n\t\n\t<h3>Configuration:</h3>\n\t<pre><code>{\n  <span class="key">"wtv":</span> <span class="number">3</span>\n}</code></pre>'), preview);
	});
});
