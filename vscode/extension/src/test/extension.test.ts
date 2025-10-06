import * as assert from 'assert';

import * as vscode from 'vscode';
import { buildOptifyPreviewData } from '../extension';
import { findOptifyRoot } from '../path-utils';
import path from 'path';

suite('Extension Test Suite', () => {
	vscode.window.showInformationMessage('Start all tests.');

	test('buildOptifyPreviewData', () => {
		const expectedRoot = path.join(__dirname, '../../src/test/configs');
		const featurePath = path.join(expectedRoot, 'feature.json');
		const optifyRoot = findOptifyRoot(featurePath, 'wtv');
		assert.equal(optifyRoot, expectedRoot);
		const previewData = buildOptifyPreviewData(['feature'], expectedRoot);

		assert.ok(!('error' in previewData), 'Preview data should not contain an error');
		assert.deepEqual(previewData.features, ['feature']);
		assert.equal(previewData.isUnsaved, false);
		assert.ok(previewData.config, 'Config should be present');
		assert.equal(previewData.config.wtv, 3);
	});
});
