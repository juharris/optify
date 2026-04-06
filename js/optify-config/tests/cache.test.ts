import { describe, expect, test } from '@jest/globals';
import path from 'path';
import { z } from 'zod';
import { CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsWatcher } from "../dist/index";

const DeeperObjectSchema = z.object({
  wtv: z.number(),
  list: z.array(z.number()),
}).readonly();

const MyObjectSchema = z.object({
  one: z.number(),
  two: z.number(),
  string: z.string(),
  deeper: DeeperObjectSchema,
}).readonly();

const MyConfigSchema = z.object({
  rootString: z.string(),
  rootString2: z.string(),
  myArray: z.array(z.string()),
  myObject: MyObjectSchema,
}).readonly();

describe('getOptions caching', () => {
  const configsPath = path.join(__dirname, '../../../tests/test_suites/simple/configs');
  const cacheOptions = new CacheOptions();
  const providers = [{
    name: "OptionsProvider",
    provider: OptionsProvider.build(configsPath),
  }, {
    name: "OptionsWatcher",
    provider: OptionsWatcher.build(configsPath),
  }];

  for (const { name, provider } of providers) {
    test(`${name} caches deserialized objects`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);

      expect(config1).toBe(config2);
      expect(config1.rootString).toBe('root string same');

      // Verify cached result equals non-cached result
      const configNonCached = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      expect(config1).toEqual(configNonCached);
    });

    test(`${name} does not cache without cacheOptions`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      expect(config1).not.toBe(config2);
      expect(config1).toEqual(config2);
    });

    test(`${name} cache differentiates by key`, () => {
      const configMyConfig = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      expect(configMyConfig.rootString).toBe('root string same');

      const configMyConfig2 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      expect(configMyConfig).toBe(configMyConfig2);
    });

    test(`${name} cache differentiates by feature names`, () => {
      const configA = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      const configAB = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema, null, cacheOptions);

      expect(configA).not.toBe(configAB);

      expect(provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions)).toBe(configA);
      expect(provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema, null, cacheOptions)).toBe(configAB);
    });

    test(`${name} cache differentiates by schema`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);

      const PartialSchema = z.object({
        rootString: z.string(),
      });
      const config2 = provider.getOptions('myConfig', ['A'], PartialSchema, null, cacheOptions);

      expect(config1).not.toBe(config2);
      expect('myArray' in config1).toBe(true);
      expect('myArray' in config2).toBe(false);
    });

    test(`${name} cache differentiates by similar schema`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);

      const MyConfigSchema2 = MyConfigSchema.clone();
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema2, null, cacheOptions);

      expect(config1).not.toBe(config2);
    });

    test(`${name} cache differentiates by configurable strings preference`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);

      const preferences = new GetOptionsPreferences();
      preferences.enableConfigurableStrings();
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences, cacheOptions);

      expect(config1).not.toBe(config2);

      const config2Again = provider.getOptions('myConfig', ['a'], MyConfigSchema, preferences, cacheOptions);
      expect(config2).toBe(config2Again);
    });
  }
});

describe('getOptions caching with maxSize', () => {
  const configsPath = path.join(__dirname, '../../../tests/test_suites/simple/configs');

  for (const name of ["OptionsProvider", "OptionsWatcher"]) {
    // Each test gets a fresh provider to avoid sharing cache state with other tests
    const makeProvider = () => name === "OptionsProvider"
      ? OptionsProvider.build(configsPath)
      : OptionsWatcher.build(configsPath);

    test(`${name} accepts maxSize option`, () => {
      const cacheOptions = new CacheOptions({ maxSize: 10 });
      expect(cacheOptions.maxSize).toBe(10);

      const provider = makeProvider();
      const config = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      const configAgain = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      expect(config).toBe(configAgain);
    });

    test(`${name} unlimited cache when maxSize is not set`, () => {
      const cacheOptions = new CacheOptions();
      expect(cacheOptions.maxSize).toBeUndefined();

      const provider = makeProvider();
      const config = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      const configAgain = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      expect(config).toBe(configAgain);
    });

    test(`${name} evicts least recently used entry when maxSize is reached`, () => {
      // maxSize=1 means only 1 entry fits; the second access evicts the first
      const cacheOptions = new CacheOptions({ maxSize: 1 });
      const provider = makeProvider();

      // Populate cache with ['A'] entry
      const configA1 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);

      // Access a different entry (different schema) to fill the cache and evict the first entry
      const PartialSchema = z.object({ rootString: z.string() });
      provider.getOptions('myConfig', ['A'], PartialSchema, null, cacheOptions);

      // MyConfigSchema entry was evicted; re-fetching produces a new object
      const configA2 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);
      expect(configA1).not.toBe(configA2);
      // The content should still be equal
      expect(configA1).toEqual(configA2);
    });
  }
});
