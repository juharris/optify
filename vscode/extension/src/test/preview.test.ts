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

	for (const ancestorPath of ['/a~1b/c~0d/0', '/a~1b/c~0d', '/a~1b']) {
		test(`option analysis retries ancestors until ${ancestorPath} is found`, () => {
			const attemptedPaths: string[] = [];
			const attemptedPreferences: GetOptionsPreferences[] = [];
			const analysis = { origins: [] };
			const provider: Pick<OptionsWatcher, 'analyze'> = {
				analyze: (keyPath, features, preferences) => {
					attemptedPaths.push(keyPath);
					assert.deepStrictEqual(features, ['base']);
					assert.ok(preferences);
					assert.equal(preferences.areConfigurableValuesEnabled(), true);
					attemptedPreferences.push(preferences);
					if (keyPath !== ancestorPath) {
						throw new Error(`configuration property at "${keyPath}" not found.`);
					}
					return analysis;
				},
			};
			const keyPath = '/a~1b/c~0d/0/child';

			const result = buildOptionAnalysis(['feature'], provider, keyPath, {
				features: ['base'],
				overrides: '{"unsaved": true}',
			}, true);

			const paths = [keyPath, '/a~1b/c~0d/0', '/a~1b/c~0d', '/a~1b'];
			assert.deepStrictEqual(attemptedPaths, paths.slice(0, paths.indexOf(ancestorPath) + 1));
			assert.ok(attemptedPreferences.every(preferences => preferences === attemptedPreferences[0]));
			assert.deepStrictEqual(result, {
				keyPath,
				analysis,
				message: `The path could not be found, likely because it points to a value in a ConfigurableList. An ancestor was found at \`${ancestorPath}\`.`,
			});
		});
	}

	test('option analysis does not retry when the requested option is found', () => {
		const attemptedPaths: string[] = [];
		const analysis = { origins: [] };
		const provider: Pick<OptionsWatcher, 'analyze'> = {
			analyze: keyPath => {
				attemptedPaths.push(keyPath);
				return analysis;
			},
		};
		const result = buildOptionAnalysis(['feature'], provider, '/message/child', undefined, true);

		assert.deepStrictEqual(attemptedPaths, ['/message/child']);
		assert.deepStrictEqual(result, { keyPath: '/message/child', analysis });
	});

	test('option analysis does not retry when configurable values are disabled', () => {
		const attemptedPaths: string[] = [];
		const provider: Pick<OptionsWatcher, 'analyze'> = {
			analyze: keyPath => {
				attemptedPaths.push(keyPath);
				throw new Error(`configuration property at "${keyPath}" not found.`);
			},
		};
		const result = buildOptionAnalysis(['feature'], provider, '/message/child');

		assert.deepStrictEqual(attemptedPaths, ['/message/child']);
		assert.deepStrictEqual(result, {
			keyPath: '/message/child',
			error: 'Error: configuration property at "/message/child" not found.',
		});
	});

	for (const failingPath of ['/message/child', '/message']) {
		test(`option analysis stops on unrelated errors at ${failingPath}`, () => {
			const attemptedPaths: string[] = [];
			const provider: Pick<OptionsWatcher, 'analyze'> = {
				analyze: keyPath => {
					attemptedPaths.push(keyPath);
					if (keyPath === failingPath) {
						throw new Error('Analysis is not enabled');
					}
					throw new Error(`configuration property at "${keyPath}" not found.`);
				},
			};
			const result = buildOptionAnalysis(['feature'], provider, '/message/child', undefined, true);

			assert.deepStrictEqual(attemptedPaths, failingPath === '/message' ? ['/message/child', '/message'] : ['/message/child']);
			assert.deepStrictEqual(result, { keyPath: '/message/child', error: 'Error: Analysis is not enabled' });
		});
	}

	for (const keyPath of ['/missing/child', '/missing', '/']) {
		test(`option analysis stops before the unsupported root for ${keyPath}`, () => {
			const attemptedPaths: string[] = [];
			const provider: Pick<OptionsWatcher, 'analyze'> = {
				analyze: keyPath => {
					attemptedPaths.push(keyPath);
					throw new Error(`configuration property at "${keyPath}" not found.`);
				},
			};
			const result = buildOptionAnalysis(['feature'], provider, keyPath, undefined, true);
			const lastPath = keyPath === '/missing/child' ? '/missing' : keyPath;

			assert.deepStrictEqual(attemptedPaths, keyPath === '/missing/child' ? [keyPath, lastPath] : [keyPath]);
			assert.deepStrictEqual(result, {
				keyPath,
				error: `Error: configuration property at "${lastPath}" not found.`,
			});
		});
	}

	test('option analysis finds an ancestor using unsaved changes', () => {
		const result = buildOptifyOptionAnalysis(['with_import'], expectedRoot, '/message/items/0/name', {
			features: ['base'],
			overrides: JSON.stringify({ message: 'unsaved' }),
		}, true);

		assert.equal(result.keyPath, '/message/items/0/name');
		assert.equal(result.error, undefined);
		assert.equal(result.message, 'The path could not be found, likely because it points to a value in a ConfigurableList. An ancestor was found at `/message`.');
		assert.deepStrictEqual(result.analysis?.origins, [{
			overridden: [],
			path: '/message',
			source: { kind: 'overrides' },
		}]);
	});

	test('option analysis finds the configurable list for an item path', () => {
		const result = buildOptifyOptionAnalysis(['configurable_list'], expectedRoot, '/items/0/name', undefined, true);

		assert.equal(result.keyPath, '/items/0/name');
		assert.equal(result.error, undefined);
		assert.equal(result.message, 'The path could not be found, likely because it points to a value in a ConfigurableList. An ancestor was found at `/items`.');
		assert.deepStrictEqual(result.analysis?.origins, [{
			overridden: [],
			path: '/items',
			source: { kind: 'feature', lineage: ['configurable_list'] },
		}]);
	});

	test('option analysis returns an error for a missing option', () => {
		const result = buildOptifyOptionAnalysis(['with_import'], expectedRoot, '/missing');
		assert.equal(result.keyPath, '/missing');
		assert.equal(result.analysis, undefined);
		assert.match(result.error ?? '', /not found/);
	});
});
