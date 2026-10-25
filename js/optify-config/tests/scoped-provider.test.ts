import { describe, expect, jest, test } from "@jest/globals";
import path from "path";
import { CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsWatcher, ScopedProvider, TypeSchema } from "../dist/index";

const configsPath = path.join(__dirname, "../../../tests/test_suites/simple/configs");
const configurableValuesConfigsPath = path.join(__dirname, "../../../tests/test_suites/configurable_values/configs");

// Returns the options as they are so that tests can compare them.
const rawSchema: TypeSchema<any> = { parse: (data) => data };

const scopeOverrides = { myConfig: { rootString: "scope override" } };
const preferenceOverrides = { myConfig: { rootString: "preference override" } };

describe("ScopedProvider", () => {
	const registries = [
		{ name: "OptionsProvider", registry: OptionsProvider.buildFromDirectories([configsPath]) },
		{ name: "OptionsWatcher", registry: OptionsWatcher.buildFromDirectories([configsPath]) },
	];

	for (const { name, registry } of registries) {
		describe(name, () => {
			test("gets options with features and overrides", () => {
				for (const features of [["feature_A"], ["feature_A", "feature_B/initial"], ["feature_B/initial", "feature_A"]]) {
					for (const overrides of [undefined, null, scopeOverrides]) {
						const scoped = new ScopedProvider(registry, features, overrides);
						const preferences = new GetOptionsPreferences();
						if (overrides) {
							preferences.setOverrides(overrides);
						}
						expect(scoped.getOptions("myConfig", rawSchema)).toEqual(registry.getOptions("myConfig", features, rawSchema, preferences));
						expect(scoped.getAllOptions()).toEqual(registry.getAllOptions(features, preferences));
					}
				}
			});

			test("scopes do not affect each other or the registry", () => {
				const original = registry.getOptions("myConfig", ["feature_A"], rawSchema);
				const scoped = new ScopedProvider(registry, ["feature_A"], scopeOverrides);
				const other = new ScopedProvider(registry, ["feature_A"]);
				expect(scoped.getOptions("myConfig", rawSchema).rootString).toBe("scope override");
				expect(other.getOptions("myConfig", rawSchema)).toEqual(original);
				expect(registry.getOptions("myConfig", ["feature_A"], rawSchema)).toEqual(original);
			});

			test("copies the given features and overrides", () => {
				const features = ["feature_A"];
				const overrides = { myConfig: { rootString: "scope override" } };
				const scoped = new ScopedProvider(registry, features, overrides);
				features.push("feature_B/initial");
				overrides.myConfig.rootString = "changed";

				expect(scoped.serialize()).toEqual({ features: ["feature_A"], overrides: scopeOverrides });
				expect(scoped.getOptions("myConfig", rawSchema).rootString).toBe("scope override");

				const serialized = scoped.serialize();
				serialized.features.push("feature_B/initial");
				(serialized.overrides as typeof overrides).myConfig.rootString = "changed";
				expect(scoped.serialize()).toEqual({ features: ["feature_A"], overrides: scopeOverrides });
			});

			test("supports empty features and overrides only", () => {
				expect(() => new ScopedProvider(registry, []).getOptions("missing", rawSchema)).toThrow();
				expect(new ScopedProvider(registry, []).getAllOptions()).toEqual(registry.getAllOptions([]));

				const overridesOnly = new ScopedProvider(registry, [], { custom: { value: 42 } });
				expect(overridesOnly.getOptions("custom", rawSchema)).toEqual({ value: 42 });
				expect(overridesOnly.getAllOptions()).toEqual({ custom: { value: 42 } });
			});

			test("propagates registry errors", () => {
				const scoped = new ScopedProvider(registry, ["feature_A"]);
				expect(() => scoped.getOptions("missing", rawSchema)).toThrow();
				expect(() => new ScopedProvider(registry, ["unknown feature"]).getAllOptions()).toThrow();
			});

			test("assumes feature names are canonical", () => {
				// "a" is an alias of "feature_A".
				expect(() => new ScopedProvider(registry, ["a"]).getOptions("myConfig", rawSchema)).toThrow();
				expect(registry.getOptions("myConfig", ["a"], rawSchema)).toEqual(registry.getOptions("myConfig", ["feature_A"], rawSchema));
			});

			describe("preferences", () => {
				test("are used when given and are not changed", () => {
					const scoped = new ScopedProvider(registry, ["feature_A"]);
					const preferences = new GetOptionsPreferences();
					preferences.setOverrides(preferenceOverrides);
					expect(scoped.getOptions("myConfig", rawSchema, preferences).rootString).toBe("preference override");
					expect(scoped.getAllOptions(preferences).myConfig.rootString).toBe("preference override");
					expect(preferences.hasOverrides()).toBe(true);
				});

				test("are copied with the scope's overrides which take precedence", () => {
					const scoped = new ScopedProvider(registry, ["feature_A"], scopeOverrides);
					const withOverrides = new GetOptionsPreferences();
					withOverrides.setOverrides(preferenceOverrides);
					const withoutOverrides = new GetOptionsPreferences();

					expect(scoped.getOptions("myConfig", rawSchema, withOverrides).rootString).toBe("scope override");
					expect(scoped.getAllOptions(withOverrides).myConfig.rootString).toBe("scope override");
					expect(scoped.getOptions("myConfig", rawSchema, withoutOverrides).rootString).toBe("scope override");

					// The given preferences are unchanged.
					expect(registry.getAllOptions(["feature_A"], withOverrides).myConfig.rootString).toBe("preference override");
					expect(withoutOverrides.hasOverrides()).toBe(false);
				});
			});

			describe("caching", () => {
				test("caches parsed options by key without using the registry cache", () => {
					const scoped = new ScopedProvider(registry, ["feature_A"], scopeOverrides);
					const schema = { parse: jest.fn(() => ({ rootString: "parsed" })) };
					const getOptions = jest.spyOn(scoped, "_getOptions");

					const first = scoped.getOptions("myConfig", schema);
					const second = scoped.getOptions("myConfig", schema);
					expect(second).toBe(first);
					expect(schema.parse).toHaveBeenCalledTimes(1);
					expect(getOptions).toHaveBeenCalledTimes(1);
				});

				test("caches when there are no overrides", () => {
					const scoped = new ScopedProvider(registry, ["feature_A"]);
					const cacheOptions = new CacheOptions();
					const options = scoped.getOptions("myConfig", rawSchema, null, cacheOptions);
					expect(scoped.getOptions("myConfig", rawSchema, null, cacheOptions)).toBe(options);
					expect(scoped.getOptions("myConfig", rawSchema)).not.toBe(options);
					expect(scoped.getOptions("myConfig", rawSchema)).toEqual(options);

					const allOptions = scoped.getAllOptions(null, cacheOptions);
					expect(scoped.getAllOptions(null, cacheOptions)).toBe(allOptions);
				});

				test("uses the registry's cache without copying preferences that do not need the scope's overrides", () => {
					const scoped = new ScopedProvider(registry, ["feature_A"]);
					expect(scoped._registry()).toBe(registry);
					expect(scoped._preferences(new GetOptionsPreferences())).toBeNull();

					const withOverrides = new ScopedProvider(registry, ["feature_A"], scopeOverrides);
					const preferences = new GetOptionsPreferences();
					const copy = withOverrides._preferences(preferences);
					expect(copy).not.toBeNull();
					expect(copy).not.toBe(preferences);
					expect(copy!.hasOverrides()).toBe(true);
					expect(preferences.hasOverrides()).toBe(false);
				});

				test("caches with the given preferences", () => {
					const scoped = new ScopedProvider(registry, ["feature_A"]);
					const cacheOptions = new CacheOptions();
					const preferences = new GetOptionsPreferences();
					const options = scoped.getOptions("myConfig", rawSchema, preferences, cacheOptions);
					expect(scoped.getOptions("myConfig", rawSchema, preferences, cacheOptions)).toBe(options);
				});

				test("does not mix up scopes with different features", () => {
					const cacheOptions = new CacheOptions();
					const a = new ScopedProvider(registry, ["feature_A"]).getOptions("myConfig", rawSchema, null, cacheOptions);
					const b = new ScopedProvider(registry, ["feature_B/initial"]).getOptions("myConfig", rawSchema, null, cacheOptions);
					expect(a).not.toEqual(b);
					expect(a).toEqual(new ScopedProvider(registry, ["feature_A"]).getOptions("myConfig", rawSchema));
					expect(b).toEqual(new ScopedProvider(registry, ["feature_B/initial"]).getOptions("myConfig", rawSchema));
				});

				test("is not supported when there are overrides", () => {
					const scoped = new ScopedProvider(registry, ["feature_A"], scopeOverrides);
					const cacheOptions = new CacheOptions();
					expect(() => scoped.getOptions("myConfig", rawSchema, null, cacheOptions)).toThrow(
						"Caching when overrides are given is not supported",
					);
					expect(() => scoped.getAllOptions(null, cacheOptions)).toThrow("Caching when overrides are given is not supported");
				});
			});

			describe("serialization", () => {
				test("serializes and restores a scope", () => {
					for (const overrides of [undefined, scopeOverrides]) {
						const scoped = new ScopedProvider(registry, ["feature_A", "feature_B/initial"], overrides);
						const value = scoped.serialize();
						expect(value).toEqual(
							overrides === undefined
								? { features: ["feature_A", "feature_B/initial"] }
								: { features: ["feature_A", "feature_B/initial"], overrides },
						);
						// `overrides` is omitted instead of being set to `undefined` or `null`.
						expect("overrides" in value).toBe(overrides !== undefined);

						const restored = ScopedProvider.deserialize(registry, JSON.parse(JSON.stringify(value)));
						expect(restored.serialize()).toEqual(value);
						expect(restored.getOptions("myConfig", rawSchema)).toEqual(scoped.getOptions("myConfig", rawSchema));
						expect(restored.getAllOptions()).toEqual(scoped.getAllOptions());
					}
				});

				test("does not require overrides to deserialize", () => {
					for (const value of [{ features: ["feature_A"] }, { features: ["feature_A"], overrides: null }]) {
						const scoped = ScopedProvider.deserialize(registry, value);
						expect(scoped.getOptions("myConfig", rawSchema)).toEqual(registry.getOptions("myConfig", ["feature_A"], rawSchema));
						expect(scoped.serialize()).toEqual({ features: ["feature_A"] });
					}
				});

				test("rejects invalid serialized scopes", () => {
					for (const value of [null, undefined, "a", 1, [], {}, { features: "a" }, { features: [1] }, { features: null }]) {
						expect(() => ScopedProvider.deserialize(registry, value)).toThrow();
					}
				});
			});
		});
	}

	test("rejects a registry that is not a provider or watcher", () => {
		const message = "The registry must be an OptionsProvider or an OptionsWatcher.";
		expect(() => new ScopedProvider({} as any, ["feature_A"])).toThrow(message);
		expect(() => new ScopedProvider(null as any, ["feature_A"])).toThrow(message);
		// Another native class must not be treated as a provider.
		expect(() => new ScopedProvider(new GetOptionsPreferences() as any, ["feature_A"])).toThrow(message);
		expect(() => ScopedProvider.deserialize(new GetOptionsPreferences() as any, { features: ["feature_A"] })).toThrow(message);
	});

	test("enables configurable values by default", () => {
		const registry = OptionsProvider.build(configurableValuesConfigsPath);
		const scoped = new ScopedProvider(registry, ["simple"]);
		expect(scoped.getOptions("greeting", rawSchema)).toBe("Hello, World!");
		expect(scoped.getAllOptions().greeting).toBe("Hello, World!");

		const disabled = new GetOptionsPreferences();
		expect(scoped.getOptions("greeting", rawSchema, disabled)).toEqual(registry.getOptions("greeting", ["simple"], rawSchema, disabled));
		expect(scoped.getOptions("greeting", rawSchema, disabled)).not.toBe("Hello, World!");
	});
});
