import * as assert from 'assert';
import path from 'path';
import { GetOptionsPreferences, OptionsWatcher } from '@optify/config';

import { buildOptifyGraphData } from '../preview/graph-data';
import { buildOptionAnalysis, buildOptifyOptionAnalysis } from '../preview/option-analysis';
import { buildOptifyPreviewData } from '../preview/preview-data';
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

		// Configurable values default to disabled
		assert.equal(result.areConfigurableValuesEnabled, false);
		assert.equal(result.areConfigurableValuesEnabledDefault, false);
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

		assert.equal(result.areConfigurableValuesEnabled, true);
		assert.equal(result.areConfigurableValuesEnabledDefault, true);
		assert.equal(result.configurableValuesError, undefined);
		assert.equal(result.configurableValuesMessage, undefined);
	});

	test('preview falls back when configurable strings cannot be rendered', () => {
		const result = buildOptifyPreviewData(['feature'], fallbackRoot, undefined, true, true);
		assert.ok(!('error' in result));

		// Configurable strings are disabled automatically after a render failure.
		assert.equal(result.areConfigurableValuesEnabled, false);
		assert.equal(result.areConfigurableValuesEnabledDefault, true);
		assert.ok(result.configurableValuesError);
		assert.ok(result.configurableValuesMessage);
	});

	test('option analysis shows the feature that set an option', () => {
		const result = buildOptifyOptionAnalysis(['with_import'], expectedRoot, '/shared/overridden');
		assert.equal(result.error, undefined);
		assert.deepStrictEqual(result, {
			keyPath: '/shared/overridden',
			analysis: {
				origins: [{
					overridden: [{ kind: 'feature', lineage: ['with_import', 'base'] }],
					path: '/shared/overridden',
					source: { kind: 'feature', lineage: ['with_import'] },
				}],
			},
		});
	});

	test('option analysis shows the chain of imports', () => {
		const result = buildOptifyOptionAnalysis(['with_import'], expectedRoot, '/shared/fromBase');
		assert.deepStrictEqual(result.analysis?.origins.map(origin => origin.source), [
			{ kind: 'feature', lineage: ['with_import', 'base'] },
		]);
	});

	test('option analysis uses unsaved changes', () => {
		const result = buildOptifyOptionAnalysis(['with_import'], expectedRoot, '/shared/overridden', {
			features: ['base'],
			overrides: JSON.stringify({ shared: { overridden: 'unsaved' } }),
		});
		assert.deepStrictEqual(result.analysis?.origins, [{
			overridden: [{ kind: 'feature', lineage: ['base'] }],
			path: '/shared/overridden',
			source: { kind: 'overrides' },
		}]);
	});

	test('option analysis retries without configurable values when the option is not found', () => {
		const configurableValuesEnabled: boolean[] = [];
		const provider = {
			analyze: (_keyPath: string, _features: string[], preferences?: GetOptionsPreferences | null) => {
				const enabled = preferences?.areConfigurableValuesEnabled() ?? false;
				configurableValuesEnabled.push(enabled);
				if (enabled) {
					throw new Error('configuration property at "/message" not found.');
				}
				return { blocker: undefined, origins: [] };
			},
		} as unknown as OptionsWatcher;

		const result = buildOptionAnalysis(['feature'], provider, '/message', undefined, true);

		assert.deepStrictEqual(configurableValuesEnabled, [true, false]);
		assert.deepStrictEqual(result, { keyPath: '/message', analysis: { blocker: undefined, origins: [] } });
	});

	test('option analysis returns an error for a missing option', () => {
		const result = buildOptifyOptionAnalysis(['with_import'], expectedRoot, '/missing');
		assert.equal(result.keyPath, '/missing');
		assert.equal(result.analysis, undefined);
		assert.match(result.error ?? '', /not found/);
	});
});
