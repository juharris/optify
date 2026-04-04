import { describe, expect, test } from '@jest/globals';
import path from 'path';
import { z } from 'zod';
import { GetOptionsPreferences, OptionsProvider, OptionsWatcher } from "../dist/index";

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
  const providers = [{
    name: "OptionsProvider",
    provider: OptionsProvider.build(configsPath),
  }, {
    name: "OptionsWatcher",
    provider: OptionsWatcher.build(configsPath),
  }];

  for (const { name, provider } of providers) {
    test(`${name} caches deserialized objects`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      expect(config1).toBe(config2);
      expect(config1.rootString).toBe('root string same');
    });

    test(`${name} cache differentiates by key`, () => {
      const configMyConfig = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      expect(configMyConfig.rootString).toBe('root string same');

      const configMyConfig2 = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      expect(configMyConfig).toBe(configMyConfig2);
    });

    test(`${name} cache differentiates by feature names`, () => {
      const configA = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      const configAB = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema);

      expect(configA).not.toBe(configAB);

      expect(provider.getOptions('myConfig', ['A'], MyConfigSchema)).toBe(configA);
      expect(provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema)).toBe(configAB);
    });

    test(`${name} cache respects feature order`, () => {
      const configAB = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema);
      const configBA = provider.getOptions('myConfig', ['B', 'A'], MyConfigSchema);

      const configAB2 = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema);
      const configBA2 = provider.getOptions('myConfig', ['B', 'A'], MyConfigSchema);

      expect(configAB).toBe(configAB2);
      expect(configBA).toBe(configBA2);
    });

    test(`${name} cache differentiates by schema`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      const PartialSchema = z.object({
        rootString: z.string(),
      });
      const config2 = provider.getOptions('myConfig', ['A'], PartialSchema);

      expect(config1).not.toBe(config2);
      expect('myArray' in config1).toBe(true);
      expect('myArray' in config2).toBe(false);
    });

    test(`${name} cache differentiates by configurable strings preference`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      const preferences = new GetOptionsPreferences();
      preferences.enableConfigurableStrings();
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences);

      expect(config1).not.toBe(config2);

      const config2Again = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences);
      expect(config2).toBe(config2Again);
    });
  }
});
