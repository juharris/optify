import * as assert from 'assert';
import path from 'path';

import { buildOptifyPreviewData, buildOptifyGraphData } from '../extension';
import { findOptifyRoot } from '../path-utils';

const expectedRoot = path.join(__dirname, '../../src/test/configs');
const fallbackRoot = path.join(__dirname, '../../src/test/configs/configurable_string_missing_arguments');

suite('Preview Builder Test Suite', () => {
	test('preview error', () => {
		const result = buildOptifyPreviewData(['nonexistent-feature'], '/nonexistent/path');
		assert.ok('error' in result, 'Should return an error for invalid path');
	});

	test('preview data', () => {
		const featurePath = path.join(expectedRoot, 'feature.json');
		const optifyRoot = findOptifyRoot(featurePath, 'wtv');
		assert.equal(optifyRoot, expectedRoot);

		const result = buildOptifyPreviewData(['feature'], expectedRoot);
		assert.ok(!('error' in result), 'Preview data should not contain an error');

		// Config
		assert.deepEqual(result.features, ['feature']);
		assert.equal(result.isUnsaved, false);
		assert.ok(result.config, 'Config should be present');
		assert.equal(typeof result.config, 'object');
		assert.equal(result.config.wtv, 3);
		assert.ok(Object.keys(result.config).length > 0);

		// Feature metadata
		assert.ok(Array.isArray(result.allFeatureNames));
		assert.ok(result.allFeatureNames.includes('feature'));
		assert.equal(typeof result.featureAliases, 'object');
		assert.equal(typeof result.featurePaths, 'object');

		// Configurable strings default to disabled
		assert.equal(result.areConfigurableStringsEnabled, false);
		assert.equal(result.areConfigurableStringsEnabledDefault, false);
	});

	test('graph data', () => {
		const graphData = buildOptifyGraphData(['feature'], expectedRoot);
		assert.ok(graphData, 'Graph data should be present');
		assert.ok(Array.isArray(graphData.nodes));
		assert.ok(Array.isArray(graphData.edges));
		assert.ok(graphData.nodes.length > 0);
		const featureNode = graphData.nodes.find(n => n.id === 'feature');
		assert.ok(featureNode);
		assert.equal(featureNode.isEnabled, true);
	});

	test('graph data returns null for invalid path', () => {
		const graphData = buildOptifyGraphData(['nonexistent-feature'], '/nonexistent/path');
		assert.equal(graphData, null);
	});

	test('preview data respects configurable strings parameters', () => {
		const result = buildOptifyPreviewData(['feature'], expectedRoot, undefined, true, true);
		assert.ok(!('error' in result));

		assert.equal(result.areConfigurableStringsEnabled, true);
		assert.equal(result.areConfigurableStringsEnabledDefault, true);
		assert.equal(result.configurableStringsError, undefined);
		assert.equal(result.configurableStringsStatusMessage, undefined);
	});

	test('preview falls back when configurable strings cannot be rendered', () => {
		const result = buildOptifyPreviewData(['feature'], fallbackRoot, undefined, true, true);
		assert.ok(!('error' in result));

		// Configurable strings are disabled automatically after a render failure.
		assert.equal(result.areConfigurableStringsEnabled, false);
		assert.equal(result.areConfigurableStringsEnabledDefault, true);
		assert.ok(result.configurableStringsError);
		assert.ok(result.configurableStringsStatusMessage);
	});
});
