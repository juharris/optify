import * as assert from 'assert';
import path from 'path';

import { buildOptifyPreviewData } from '../extension';
import { findOptifyRoot } from '../path-utils';

suite('Preview Builder Test Suite', () => {
	test('preview error', () => {
		const result = buildOptifyPreviewData(['nonexistent-feature'], '/nonexistent/path');
		assert.ok('error' in result, 'Should return an error for invalid path');
	});

	test('preview data should contain config options', () => {
		const expectedRoot = path.join(__dirname, '../../src/test/configs');
		const featurePath = path.join(expectedRoot, 'feature.json');
		const optifyRoot = findOptifyRoot(featurePath, 'wtv');
		assert.equal(optifyRoot, expectedRoot);

		const result = buildOptifyPreviewData(['feature'], expectedRoot);
		assert.ok(!('error' in result), 'Preview data should not contain an error');

		assert.deepEqual(result.features, ['feature']);
		assert.equal(result.isUnsaved, false);
		assert.ok(result.config, 'Config should be present');
		assert.equal(typeof result.config, 'object', 'Config should be an object');
		assert.equal(result.config.wtv, 3, 'Config should contain wtv option with value 3');
		assert.ok(Object.keys(result.config).length > 0, 'Config should not be empty');
	});
});
