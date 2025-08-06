import { describe, expect, test } from '@jest/globals';
import fs from 'fs';
import path from 'path';
import { OptionsProvider, OptionsWatcher } from "../dist/index";

const configsPath = path.join(__dirname, '../../../tests/test_suites/simple/configs')
const expectationsPath = path.join(configsPath, '../expectations')

describe('Provider', () => {
	const start = Date.now()
	const provider = OptionsProvider.buildFromDirectories([configsPath])
	const watcher = OptionsWatcher.buildFromDirectories([configsPath])
	const providers = [{
		name: "OptionsProvider",
		provider: provider,
	}, {
		name: "OptionsWatcher",
		provider: watcher
	}]

	for (const { name, provider } of providers) {
		test(`${name} get_all_options_json feature_A`, () => {
			const options = JSON.parse(provider.getAllOptionsJson(['feature_A']))
			const expectedOptions = JSON.parse(fs.readFileSync(path.join(configsPath, 'feature_A.json'), 'utf8'))['options']
			expect(options).toEqual(expectedOptions)
		})

		test(`${name} get_all_options_json A and B`, () => {
			const options = JSON.parse(provider.getAllOptionsJson(['A', 'B']))
			const expectedOptions = JSON.parse(fs.readFileSync(path.join(expectationsPath, 'aliases.json'), 'utf8'))['options']
			expect(options).toEqual(expectedOptions)
		})

		test(`${name} invalid file`, () => {
			const configDir = path.relative(__dirname, path.join(__dirname, '../../rust/optify/tests/invalid_file'))
			const relativePath = path.join(configDir, 'invalid.yaml')
			const absolutePath = path.resolve(relativePath)
			expect(() => OptionsProvider.build(configDir))
				.toThrow(`Error loading file '${absolutePath}': simple key expected at byte 31 line 4 column 1 in ${relativePath}`)
		})

		test(`${name} features`, () => {
			const features = provider.features()
			features.sort()
			expect(features).toEqual(['A_with_comments', 'feature_A', 'feature_B/initial'])
		})

		test(`${name} features with metadata`, () => {
			const featuresWithMetadata = provider.featuresWithMetadata()
			const keys = Object.keys(featuresWithMetadata)
			keys.sort()
			expect(keys).toEqual(['A_with_comments', 'feature_A', 'feature_B/initial'])
			const metadataA = featuresWithMetadata['feature_A']
			expect(metadataA.name()).toEqual('feature_A')
			expect(metadataA.aliases()).toEqual(['a'])
			expect(metadataA.owners()).toEqual("a-team@company.com")
			const expectedPath = path.resolve(path.join(configsPath, 'feature_A.json'))
			expect(metadataA.path()).toEqual(expectedPath)

			const secondCall = provider.featuresWithMetadata()
			expect(secondCall).toBe(featuresWithMetadata)
		})

		test(`${name} get_canonical_feature_name`, () => {
			expect(provider.getCanonicalFeatureName('a')).toEqual('feature_A')
			expect(provider.getCanonicalFeatureName('A')).toEqual('feature_A')
			expect(provider.getCanonicalFeatureName('feature_A')).toEqual('feature_A')
			expect(provider.getCanonicalFeatureName('feAture_B/InItiAl')).toEqual('feature_B/initial')
			expect(provider.getCanonicalFeatureName('B')).toEqual('feature_B/initial')
		})

		test(`${name} get_canonical_feature_names does not exist`, () => {
			expect(provider.getCanonicalFeatureName('does_not_exist')).toBeNull()
		})
	}

	test("last modified", () => {
		const lastModified = watcher.lastModified()
		expect(lastModified).toBeGreaterThan(start)
	})
})