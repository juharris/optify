import { describe, expect, test } from "@jest/globals";
import fs from "fs";
import path from "path";
import {
	BuilderOptions,
	Feature,
	FeatureMetadata,
	GetOptionsPreferences,
	OptionsMetadata,
	OptionsProvider,
	OptionsProviderBuilder,
	OptionsWatcher,
	OptionsWatcherBuilder,
	Policies,
} from "../dist/index";

const configsPath = path.join(__dirname, "../../../tests/test_suites/simple/configs");
const conditionsConfigsPath = path.join(__dirname, "../../../tests/test_suites/conditions/configs");
const policiesConfigsPath = path.join(__dirname, "../../../tests/test_suites/policies/configs");
const inheritanceConfigsPath = path.join(__dirname, "../../../tests/test_suites/inheritance/configs");
const analysisConfigsPath = path.join(__dirname, "../../../rust/optify/tests/analysis");
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

describe("builder features", () => {
	const feature = (name: string, value: string): Feature => ({
		metadata: new OptionsMetadata({
			aliases: [`${name}_alias`],
			details: { source: "memory" },
			name,
		}),
		options: { builderConfig: { value } },
	});

	const builders = [
		{ name: "OptionsProviderBuilder", create: () => new OptionsProviderBuilder() },
		{ name: "OptionsWatcherBuilder", create: () => new OptionsWatcherBuilder() },
	];

	for (const { name, create } of builders) {
		test(`${name} accepts object metadata and features parsed with JSON.parse`, () => {
			const metadata: FeatureMetadata = { name: "my_feat", aliases: ["object_alias"], details: { source: "object" } };
			const objectFeature: Feature = { metadata, options: { myConfig: { key: "my_value" } } };
			const parsed: Feature = JSON.parse(
				'{"metadata":{"name":"from_json","aliases":["json_alias"]},"options":{"myConfig":{"key":"json_value"}}}',
			);
			const builder = create();
			builder.addFeature(objectFeature);
			builder.addFeatures([parsed]);
			builder.addFeature({ metadata: { name: "inline" }, options: { myConfig: { key: "inline_value" } } });

			const provider = builder.build();
			expect(provider.getAllOptions(["object_alias"])).toEqual({ myConfig: { key: "my_value" } });
			expect(provider.featuresWithMetadata()["my_feat"].details()).toEqual({ source: "object" });
			expect(provider.getAllOptions(["json_alias"])).toEqual({ myConfig: { key: "json_value" } });
			expect(provider.getAllOptions(["inline"])).toEqual({ myConfig: { key: "inline_value" } });
		});

		test(`${name} adds a feature deserialized from JSON, including metadata`, () => {
			const json = `{
				"metadata": {
					"name": "from_json",
					"aliases": ["json_alias"],
					"details": { "source": "json" },
					"owners": "team@example.com",
					"path": "features/from_json.json"
				},
				"options": { "builderConfig": { "value": "from JSON", "metadata": { "nested": true } } },
				"imports": ["base"],
				"conditions": { "jsonPointer": "/environment", "equals": "test" },
				"policies": { "requester": { "allow": ["service_a"] } }
			}`;
			const parsed = Feature.fromJson(json);

			expect(parsed.metadata).toBeInstanceOf(OptionsMetadata);
			expect(parsed.metadata.name()).toBe("from_json");
			expect(parsed.metadata.aliases()).toEqual(["json_alias"]);
			expect(parsed.metadata.details()).toEqual({ source: "json" });
			expect(parsed.metadata.owners()).toBe("team@example.com");
			expect(parsed.metadata.path()).toBe("features/from_json.json");

			for (const add of [
				(builder: ReturnType<typeof create>) => builder.addFeature(parsed),
				(builder: ReturnType<typeof create>) => builder.addFeatures([parsed]),
			]) {
				const builder = create();
				builder.addFeature(feature("base", "base"));
				add(builder);
				const provider = builder.build();
				const preferences = new GetOptionsPreferences();
				preferences.setConstraints({ environment: "test" });

				expect(provider.getAllOptions(["json_alias"], preferences)).toEqual({
					builderConfig: { value: "from JSON", metadata: { nested: true } },
				});
				expect(provider.getPolicies("from_json")).toEqual({ requester: { allow: ["service_a"] } });
			}
		});

		test(`${name} adds features with metadata, imports, conditions, and policies`, () => {
			const builder = create();
			builder.addFeature({
				...feature("base", "first"),
				options: { builderConfig: { inherited: true, value: "first" } },
			});
			builder.addFeatures([
				{
					...feature("child", "second"),
					conditions: { jsonPointer: "/environment", equals: "test" },
					imports: ["base"],
					policies: { requester: { allow: ["service_a"] } },
				},
				feature("last", "third"),
			]);

			const provider = builder.build();
			const preferences = new GetOptionsPreferences();
			preferences.setConstraints({ environment: "test" });

			expect(provider.features().sort()).toEqual(["base", "child", "last"]);
			expect(provider.getAllOptions(["child"], preferences)).toEqual({
				builderConfig: { inherited: true, value: "second" },
			});
			expect(provider.getAllOptions(["base", "last"])).toEqual({ builderConfig: { inherited: true, value: "third" } });
			expect(provider.getFilteredFeatures(["child"], preferences)).toEqual(["child"]);
			preferences.setConstraints({ environment: "production" });
			expect(provider.getFilteredFeatures(["child"], preferences)).toEqual([]);
			expect(provider.getPolicies("child")).toEqual({ requester: { allow: ["service_a"] } });
			expect(provider.getCanonicalFeatureName("base_alias")).toBe("base");
			const metadata = provider.featuresWithMetadata()["base"];
			expect(metadata).toBeInstanceOf(OptionsMetadata);
			expect(metadata.details()).toEqual({ source: "memory" });
			expect(metadata.name()).toBe("base");
			expect(metadata.toJson()).toMatchObject({ name: "base", aliases: ["base_alias"], details: { source: "memory" } });

			const secondBuilder = create();
			secondBuilder.addFeature({ metadata, options: { reused: true } });
			expect(secondBuilder.build().getAllOptions(["base_alias"])).toEqual({ reused: true });
		});

		test(`${name} combines in-memory features with a directory`, () => {
			const builder = create();
			builder.addFeature({
				imports: ["feature_A"],
				metadata: new OptionsMetadata({ name: "memory" }),
				options: { myConfig: { rootString: "in memory" } },
			});
			builder.addDirectory(configsPath);
			const provider = builder.build();

			expect(provider.getAllOptions(["memory"]).myConfig).toMatchObject({
				rootString: "in memory",
				rootString2: "gets overridden",
			});
		});

		test(`${name} rejects invalid feature input before modifying a batch`, () => {
			const builder = create();
			expect(() => builder.addFeature({ metadata: new OptionsMetadata(), options: {} })).toThrow(/non-empty name/);
			expect(() => builder.addFeature({ metadata: new OptionsMetadata({ name: "" }), options: {} })).toThrow(/non-empty name/);
			expect(() =>
				builder.addFeature({
					// @ts-expect-error Verify that a different native class is rejected at runtime.
					metadata: new BuilderOptions(),
					options: {},
				}),
			).toThrow(/Feature\.metadata/);
			const spoofedMetadata = new BuilderOptions();
			Object.setPrototypeOf(spoofedMetadata, OptionsMetadata.prototype);
			expect(() =>
				builder.addFeature({
					// @ts-expect-error A different native class must not pass even with a spoofed prototype.
					metadata: spoofedMetadata,
					options: {},
				}),
			).toThrow(TypeError);
			expect(() =>
				builder.addFeatures([
					feature("valid", "first"),
					{
						// @ts-expect-error The whole batch must be rejected when metadata is spoofed.
						metadata: spoofedMetadata,
						options: {},
					},
				]),
			).toThrow(TypeError);
			expect(() => builder.addFeature({ metadata: {}, options: {} })).toThrow(/non-empty name/);
			expect(() =>
				builder.addFeatures([
					{ metadata: { name: "valid" }, options: {} },
					{
						// @ts-expect-error Verify malformed plain-object metadata is rejected.
						metadata: { name: "invalid", aliases: 42 },
						options: {},
					},
				]),
			).toThrow(/Invalid options metadata/);
			// @ts-expect-error Verify that missing metadata is rejected at runtime.
			expect(() => builder.addFeature({ options: {} })).toThrow(/metadata/);
			expect(() =>
				builder.addFeatures([
					feature("valid", "first"),
					{ ...feature("invalid", "second"), conditions: { jsonPointer: "/environment", matches: "[" } },
				]),
			).toThrow(/Invalid feature conditions/);
			expect(() =>
				builder.addFeature({
					...feature("invalid", "second"),
					// @ts-expect-error Verify that invalid policies are rejected at runtime.
					policies: { requester: {} },
				}),
			).toThrow(/Invalid feature policies/);
			expect(builder.build().features()).toEqual([]);
		});
	}

	test("rejects invalid feature JSON", () => {
		for (const json of [
			"{",
			"null",
			"{}",
			'{"metadata": null, "options": {}}',
			'{"metadata": {}, "options": {}}',
			'{"metadata": {"name": ""}, "options": {}}',
			'{"metadata": {"name": "invalid"}}',
			'{"metadata": {"name": "invalid", "aliases": 42}, "options": {}}',
			'{"metadata": {"name": "invalid"}, "options": {}, "conditions": {"jsonPointer": "/x", "matches": "["}}',
			'{"metadata": {"name": "invalid"}, "options": {}, "policies": {"requester": {}}}',
		]) {
			expect(() => Feature.fromJson(json)).toThrow();
		}
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

describe("getPolicies", () => {
	const providers = [
		{ name: "OptionsProvider", provider: OptionsProvider.build(policiesConfigsPath) },
		{ name: "OptionsWatcher", provider: OptionsWatcher.build(policiesConfigsPath) },
	];

	// The lists come from sets, so sort them for stable comparisons.
	const sortLists = (policies: Policies | null) => {
		const requester = policies?.requester;
		if (requester) {
			("allow" in requester ? requester.allow : requester.block).sort();
		}
		return policies;
	};

	for (const { name, provider } of providers) {
		test(`${name} returns the feature's allow list`, () => {
			expect(sortLists(provider.getPolicies("feature_allowed"))).toEqual({
				requester: { allow: ["service_a", "service_d"] },
			});
		});

		test(`${name} merges the feature's block list with requesters denied by .optify/policies.json`, () => {
			expect(sortLists(provider.getPolicies("feature_blocked"))).toEqual({
				requester: { block: ["requester_x", "service_b", "service_f", "untrusted_service"] },
			});
		});

		test(`${name} returns requesters denied only by .optify/policies.json`, () => {
			expect(sortLists(provider.getPolicies("feature_neutral"))).toEqual({
				requester: { block: ["requester_y", "service_a", "service_d"] },
			});
		});

		test(`${name} returns null when nothing restricts the feature`, () => {
			const unrestricted = OptionsProvider.build(configsPath);
			expect(unrestricted.getPolicies("feature_A")).toBeNull();
		});
	}
});

describe("analyze", () => {
	const builderOptions = new BuilderOptions();
	builderOptions.setAnalysisEnabled(true);
	const buildProviders = (directory: string) => [
		{ name: "OptionsProvider", provider: OptionsProvider.build(directory, builderOptions) },
		{ name: "OptionsWatcher", provider: OptionsWatcher.build(directory, builderOptions) },
	];

	for (const { name, provider } of buildProviders(inheritanceConfigsPath)) {
		test(`${name} explains the import chain and overridden features`, () => {
			expect(provider.analyze("/myConfig/key2", ["grandparent"])).toEqual({
				origins: [
					{
						overridden: [{ lineage: ["grandparent", "parent1"], kind: "feature" }],
						path: "/myConfig/key2",
						source: { lineage: ["grandparent", "parent2", "base2"], kind: "feature" },
					},
				],
			});
		});

		test(`${name} lists one origin per sub-path of an object`, () => {
			const analysis = provider.analyze("/myConfig", ["grandparent"]);
			expect(analysis.origins.map((origin) => [origin.path, origin.source.lineage])).toEqual([
				["/myConfig/key1", ["grandparent", "parent1", "base1"]],
				["/myConfig/key2", ["grandparent", "parent2", "base2"]],
			]);
		});

		test(`${name} throws when the path is not found`, () => {
			expect(() => provider.analyze("/missing", ["grandparent"])).toThrow(/not found/);
		});
	}

	for (const { name, provider } of buildProviders(analysisConfigsPath)) {
		test(`${name} explains a blocked path`, () => {
			expect(provider.analyze("/k/x", ["x", "null_k"])).toEqual({
				blocker: {
					overridden: [{ lineage: ["x"], kind: "feature" }],
					path: "/k",
					source: { lineage: ["null_k"], kind: "feature" },
				},
				origins: [],
			});
		});

		test(`${name} attributes overrides`, () => {
			const preferences = new GetOptionsPreferences();
			preferences.setOverridesJson(JSON.stringify({ k: { x: 99 } }));
			expect(provider.analyze("/k/x", ["x"], preferences).origins).toEqual([
				{
					overridden: [{ lineage: ["x"], kind: "feature" }],
					path: "/k/x",
					source: { kind: "overrides" },
				},
			]);
		});
	}

	test("throws when analysis is not enabled", () => {
		const provider = OptionsProvider.build(inheritanceConfigsPath);
		expect(() => provider.analyze("/myConfig/key2", ["grandparent"])).toThrow(/Analysis is not enabled/);
	});
});
