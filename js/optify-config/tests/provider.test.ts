import { describe, expect, test } from "@jest/globals";
import fs from "fs";
import path from "path";
import { GetOptionsPreferences, OptionsProvider, OptionsWatcher } from "../dist/index";

const configsPath = path.join(__dirname, "../../../tests/test_suites/simple/configs");
const conditionsConfigsPath = path.join(__dirname, "../../../tests/test_suites/conditions/configs");
const expectationsPath = path.join(configsPath, "../expectations");

describe("Provider", () => {
	const start = Date.now();
	const PROVIDER = OptionsProvider.buildFromDirectories([configsPath]);
	const WATCHER = OptionsWatcher.buildFromDirectories([configsPath]);
	const providers = [
		{
			name: "OptionsProvider",
			provider: PROVIDER,
		},
		{
			name: "OptionsWatcher",
			provider: WATCHER,
		},
	];

	test("as type", () => {
		// Ensure that we can use the overridden type in generics.
		const providerCache = new Map<string, OptionsProvider>();
		providerCache.set("a", providers[0].provider);
		const provider = providerCache.get("a");
		expect(provider).toBe(providers[0].provider);
		const features = provider!.features();
		expect(features).toHaveLength(3);
	});

	for (const { name, provider } of providers) {
		test(`${name} get_all_options_json feature_A`, () => {
			const options = JSON.parse(provider.getAllOptionsJson(["feature_A"]));
			const expectedOptions = JSON.parse(fs.readFileSync(path.join(configsPath, "feature_A.json"), "utf8"))["options"];
			expect(options).toEqual(expectedOptions);

			const optionsObj = provider.getAllOptions(["feature_A"]);
			// NOTE: `instanceof Object` is not reliable under Jest because test files run in a VM
			// context with their own `Object` constructor; values coming from native bindings may
			// be created in a different realm. Prefer structural/type checks instead.
			expect(typeof optionsObj).toBe("object");
			expect(optionsObj).toEqual(expectedOptions);
		});

		test(`${name} get_all_options_json A and B`, () => {
			const options = JSON.parse(provider.getAllOptionsJson(["A", "B"]));
			const expectedOptions = JSON.parse(fs.readFileSync(path.join(expectationsPath, "aliases.json"), "utf8"))["options"];
			expect(options).toEqual(expectedOptions);
		});

		test(`${name} invalid file`, () => {
			const configDir = path.relative(__dirname, path.join(__dirname, "../../rust/optify/tests/invalid_file"));
			const relativePath = path.join(configDir, "invalid.yaml");
			const absolutePath = path.resolve(relativePath);
			expect(() => OptionsProvider.build(configDir)).toThrow(
				`Error loading file '${absolutePath}': simple key expected at byte 31 line 4 column 1 in ${relativePath}`,
			);
		});

		test(`${name} features`, () => {
			const features = provider.features();
			features.sort();
			expect(features).toEqual(["A_with_comments", "feature_A", "feature_B/initial"]);
		});

		test(`${name} features with metadata`, () => {
			const featuresWithMetadata = provider.featuresWithMetadata();
			const keys = Object.keys(featuresWithMetadata);
			keys.sort();
			expect(keys).toEqual(["A_with_comments", "feature_A", "feature_B/initial"]);
			const metadataA = featuresWithMetadata["feature_A"];
			expect(metadataA.name()).toEqual("feature_A");
			expect(metadataA.aliases()).toEqual(["a"]);
			expect(metadataA.dependents()).toEqual(null);
			expect(metadataA.owners()).toEqual("a-team@company.com");
			const expectedPath = path.resolve(path.join(configsPath, "feature_A.json"));
			expect(metadataA.path()).toEqual(expectedPath);

			const secondCall = provider.featuresWithMetadata();
			expect(secondCall).toBe(featuresWithMetadata);
		});

		test(`${name} get_canonical_feature_name`, () => {
			expect(provider.getCanonicalFeatureName("a")).toEqual("feature_A");
			expect(provider.getCanonicalFeatureName("A")).toEqual("feature_A");
			expect(provider.getCanonicalFeatureName("feature_A")).toEqual("feature_A");
			expect(provider.getCanonicalFeatureName("feAture_B/InItiAl")).toEqual("feature_B/initial");
			expect(provider.getCanonicalFeatureName("B")).toEqual("feature_B/initial");
		});

		test(`${name} get_canonical_feature_names does not exist`, () => {
			expect(provider.getCanonicalFeatureName("does_not_exist")).toBeNull();
		});
	}

	test("last modified", () => {
		const lastModified = WATCHER.lastModified();
		expect(lastModified).toBeGreaterThan(start);
	});
});

describe("mapFeatureNames", () => {
	const conditionsProvider = OptionsProvider.buildFromDirectories([conditionsConfigsPath]);
	const conditionsWatcher = OptionsWatcher.buildFromDirectories([conditionsConfigsPath]);
	const conditionsProviders = [
		{ name: "OptionsProvider", provider: conditionsProvider },
		{ name: "OptionsWatcher", provider: conditionsWatcher },
	];

	for (const { name, provider } of conditionsProviders) {
		test(`${name} no preferences`, () => {
			const result = provider.mapFeatureNames(["a", "b"]);
			expect(result).toEqual(["A", "B"]);
		});

		test(`${name} skip feature name conversion`, () => {
			const preferences = new GetOptionsPreferences();
			preferences.setSkipFeatureNameConversion(true);
			const result = provider.mapFeatureNames(["A", "B"], preferences);
			expect(result).toEqual(["A", "B"]);
		});

		test(`${name} constraints matching all features`, () => {
			const preferences = new GetOptionsPreferences();
			preferences.setConstraints({ info: 3, status: "new" });
			const result = provider.mapFeatureNames(["a", "b"], preferences);
			expect(result).toEqual(["A", "B"]);
		});

		test(`${name} constraints filtering out a feature`, () => {
			const preferences = new GetOptionsPreferences();
			preferences.setConstraints({ info: 2, status: "new" });
			const result = provider.mapFeatureNames(["a", "b"], preferences);
			expect(result).toEqual([null, "B"]);
		});

		test(`${name} reversed input order with filtering`, () => {
			const preferences = new GetOptionsPreferences();
			preferences.setConstraints({ info: 2, status: "new" });
			const result = provider.mapFeatureNames(["b", "a"], preferences);
			expect(result).toEqual(["B", null]);
		});

		test(`${name} empty input`, () => {
			const preferences = new GetOptionsPreferences();
			preferences.setConstraintsJson(JSON.stringify({ info: 2, status: "new" }));
			const result = provider.mapFeatureNames([], preferences);
			expect(result).toEqual([]);
		});
	}
});

describe("overrides", () => {
	const provider = OptionsProvider.buildFromDirectories([configsPath]);

	test("hasOverrides is false by default", () => {
		const preferences = new GetOptionsPreferences();
		expect(preferences.hasOverrides()).toBe(false);
	});

	test("setOverrides sets overrides and hasOverrides returns true", () => {
		const preferences = new GetOptionsPreferences();
		preferences.setOverrides({ myConfig: { rootString2: "overridden value" } });
		expect(preferences.hasOverrides()).toBe(true);

		const options = provider.getAllOptions(["feature_A"], preferences);
		expect(options.myConfig.rootString2).toBe("overridden value");

		preferences.setOverrides(null);
		expect(preferences.hasOverrides()).toBe(false);
	});

	test("setOverridesJson sets overrides from a JSON string", () => {
		const preferences = new GetOptionsPreferences();
		preferences.setOverridesJson(JSON.stringify({ myConfig: { rootString2: "json override" } }));
		expect(preferences.hasOverrides()).toBe(true);

		const options = provider.getAllOptions(["feature_A"], preferences);
		expect(options.myConfig.rootString2).toBe("json override");

		preferences.setOverridesJson(null);
		expect(preferences.hasOverrides()).toBe(false);
	});

	test("overrides do not affect keys that are not overridden", () => {
		const preferences = new GetOptionsPreferences();
		preferences.setOverrides({ myConfig: { rootString2: "overridden" } });

		const options = provider.getAllOptions(["feature_A"], preferences);
		expect(options.myConfig.rootString).toBe("root string same");
		expect(options.myConfig.rootString2).toBe("overridden");
	});
});
