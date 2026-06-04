import { describe, expect, test } from "@jest/globals";
import path from "path";
import { z } from "zod";
import { CacheInitOptions, CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsWatcher } from "../dist/index";

const DeeperObjectSchema = z
	.object({
		wtv: z.number(),
		list: z.array(z.number()),
	})
	.readonly();

const MyObjectSchema = z
	.object({
		one: z.number(),
		two: z.number(),
		string: z.string(),
		deeper: DeeperObjectSchema,
	})
	.readonly();

const MyConfigSchema = z
	.object({
		rootString: z.string(),
		rootString2: z.string(),
		myArray: z.array(z.string()),
		myObject: MyObjectSchema,
	})
	.readonly();

describe("getOptions caching", () => {
	const configsPath = path.join(__dirname, "../../../tests/test_suites/simple/configs");
	const cacheOptions = new CacheOptions();
	const providers = [
		{
			name: "OptionsProvider",
			provider: OptionsProvider.build(configsPath).init(),
		},
		{
			name: "OptionsWatcher",
			provider: OptionsWatcher.build(configsPath).init(),
		},
	];

	for (const { name, provider } of providers) {
		test(`${name} caches deserialized objects`, () => {
			const config1 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			const config2 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);

			expect(config1).toBe(config2);
			expect(config1.rootString).toBe("root string same");

			const configNonCached = provider.getOptions("myConfig", ["A"], MyConfigSchema);
			expect(config1).toEqual(configNonCached);
		});

		test(`${name} does not cache without cacheOptions`, () => {
			const config1 = provider.getOptions("myConfig", ["A"], MyConfigSchema);
			const config2 = provider.getOptions("myConfig", ["A"], MyConfigSchema);

			expect(config1).not.toBe(config2);
			expect(config1).toEqual(config2);
		});

		test(`${name} cache differentiates by key`, () => {
			const configMyConfig = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			expect(configMyConfig.rootString).toBe("root string same");

			const configMyConfig2 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			expect(configMyConfig).toBe(configMyConfig2);
		});

		test(`${name} cache differentiates by feature names`, () => {
			const configA = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			const configAB = provider.getOptions("myConfig", ["A", "B"], MyConfigSchema, null, cacheOptions);

			expect(configA).not.toBe(configAB);

			expect(provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions)).toBe(configA);
			expect(provider.getOptions("myConfig", ["A", "B"], MyConfigSchema, null, cacheOptions)).toBe(configAB);
		});

		test(`${name} cache differentiates by schema`, () => {
			const config1 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);

			const PartialSchema = z.object({
				rootString: z.string(),
			});
			const config2 = provider.getOptions("myConfig", ["A"], PartialSchema, null, cacheOptions);

			expect(config1).not.toBe(config2);
			expect("myArray" in config1).toBe(true);
			expect("myArray" in config2).toBe(false);
		});

		test(`${name} cache differentiates by similar schema`, () => {
			const config1 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);

			const MyConfigSchema2 = MyConfigSchema.clone();
			const config2 = provider.getOptions("myConfig", ["A"], MyConfigSchema2, null, cacheOptions);

			expect(config1).not.toBe(config2);
		});

		test(`${name} cache differentiates by configurable strings preference`, () => {
			const config1 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);

			const preferences = new GetOptionsPreferences();
			preferences.enableConfigurableValues();
			const config2 = provider.getOptions("myConfig", ["A"], MyConfigSchema, preferences, cacheOptions);

			expect(config1).not.toBe(config2);

			const config2Again = provider.getOptions("myConfig", ["a"], MyConfigSchema, preferences, cacheOptions);
			expect(config2).toBe(config2Again);
		});
	}
});

describe("getOptions caching without init", () => {
	const configsPath = path.join(__dirname, "../../../tests/test_suites/simple/configs");
	const cacheOptions = new CacheOptions();

	for (const { name, provider } of [
		{
			name: "OptionsProvider",
			provider: OptionsProvider.build(configsPath),
		},
		{
			name: "OptionsWatcher",
			provider: OptionsWatcher.build(configsPath),
		},
	]) {
		test(`${name} caches without calling init`, () => {
			const config1 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			const config2 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);

			expect(config1).toBe(config2);

			const configNonCached = provider.getOptions("myConfig", ["A"], MyConfigSchema);
			expect(config1).toEqual(configNonCached);
			expect(config1).not.toBe(configNonCached);
		});
	}
});

describe("getAllOptions caching", () => {
	const configsPath = path.join(__dirname, "../../../tests/test_suites/simple/configs");
	const cacheOptions = new CacheOptions();
	const providers = [
		{
			name: "OptionsProvider",
			provider: OptionsProvider.build(configsPath).init(),
		},
		{
			name: "OptionsWatcher",
			provider: OptionsWatcher.build(configsPath).init(),
		},
	];

	for (const { name, provider } of providers) {
		test(`${name} caches deserialized objects`, () => {
			const options1 = provider.getAllOptions(["A"], null, cacheOptions);
			const options2 = provider.getAllOptions(["A"], null, cacheOptions);

			expect(options1).toBe(options2);

			const optionsNonCached = provider.getAllOptions(["A"]);
			expect(options1).toEqual(optionsNonCached);
			expect(options1).not.toBe(optionsNonCached);
		});

		test(`${name} does not cache without cacheOptions`, () => {
			const options1 = provider.getAllOptions(["A"]);
			const options2 = provider.getAllOptions(["A"]);

			expect(options1).not.toBe(options2);
			expect(options1).toEqual(options2);
		});

		test(`${name} throws when caching with overrides`, () => {
			const preferences = new GetOptionsPreferences();
			preferences.setOverrides({ myConfig: { rootString2: "overridden value" } });

			expect(() => provider.getAllOptions(["A"], preferences, cacheOptions)).toThrow(
				"Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.",
			);
		});
	}
});

describe("getOptions caching with maxSize", () => {
	const configsPath = path.join(__dirname, "../../../tests/test_suites/simple/configs");
	const cacheOptions = new CacheOptions();

	const builders = [
		{
			name: "OptionsProvider",
			build: () => OptionsProvider.build(configsPath),
		},
		{
			name: "OptionsWatcher",
			build: () => OptionsWatcher.build(configsPath),
		},
	];

	for (const { name, build } of builders) {
		// Each test gets a fresh provider to avoid sharing cache state with other tests
		const makeProvider = (cacheInitOptions?: CacheInitOptions) => build().init(cacheInitOptions);

		test(`${name} unlimited cache when maxSize is not set`, () => {
			const provider = makeProvider();
			const config = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			const configAgain = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			expect(config).toBe(configAgain);
		});

		test(`${name} evicts least recently used entry when maxSize is reached`, () => {
			// maxSize=1 means only 1 entry fits; the second access evicts the first
			const provider = makeProvider(new CacheInitOptions(1));

			// Populate cache with ['A'] entry
			const configA1 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);

			// Access a different entry (different schema) to fill the cache and evict the first entry
			const PartialSchema = z.object({ rootString: z.string() });
			provider.getOptions("myConfig", ["A"], PartialSchema, null, cacheOptions);

			// MyConfigSchema entry was evicted; re-fetching produces a new object
			const configA2 = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			expect(configA1).not.toBe(configA2);
			// The content should still be equal
			expect(configA1).toEqual(configA2);

			// Get the same thing and it should be the same cached object
			const configA2Again = provider.getOptions("myConfig", ["A"], MyConfigSchema, null, cacheOptions);
			expect(configA2).toBe(configA2Again);
		});
	}
});
