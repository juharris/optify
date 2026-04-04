import { describe, expect, test } from '@jest/globals';
import path from 'path';
import { z } from 'zod';
import { CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsWatcher } from "../dist/index";

const DeeperObjectSchema = z.object({
  wtv: z.number(),
  list: z.array(z.number()),
});

const MyObjectSchema = z.object({
  one: z.number(),
  two: z.number(),
  string: z.string(),
  deeper: DeeperObjectSchema,
});

const MyConfigSchema = z.object({
  rootString: z.string(),
  rootString2: z.string(),
  myArray: z.array(z.string()),
  myObject: MyObjectSchema,
});

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

    test(`${name} cache respects feature order`, () => {
      const configAB = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema, null, cacheOptions);
      const configBA = provider.getOptions('myConfig', ['B', 'A'], MyConfigSchema, null, cacheOptions);

      const configAB2 = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema, null, cacheOptions);
      const configBA2 = provider.getOptions('myConfig', ['B', 'A'], MyConfigSchema, null, cacheOptions);

      expect(configAB).toBe(configAB2);
      expect(configBA).toBe(configBA2);
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

    test(`${name} cache differentiates by configurable strings preference`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema, null, cacheOptions);

      const preferences = new GetOptionsPreferences();
      preferences.enableConfigurableStrings();
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences, cacheOptions);

      expect(config1).not.toBe(config2);

      const config2Again = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences, cacheOptions);
      expect(config2).toBe(config2Again);
    });
  }
});
