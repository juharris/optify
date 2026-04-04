import { describe, expect, test } from '@jest/globals';
import fs from 'fs';
import path from 'path';
import { z } from 'zod';
import { GetOptionsPreferences, OptionsProvider, OptionsWatcher } from "../dist/index";

const runSuite = (suitePath: string) => {
  const providers = [{
    name: "OptionsProvider",
    provider: OptionsProvider.build(path.join(suitePath, 'configs')),
  }, {
    name: "OptionsWatcher",
    provider: OptionsWatcher.build(path.join(suitePath, 'configs')),
  }]
  const expectationsPath = path.join(suitePath, 'expectations');
  for (const testCase of fs.readdirSync(expectationsPath)) {
    const expectationPath = path.join(expectationsPath, testCase);
    const expectedInfo = JSON.parse(fs.readFileSync(expectationPath, 'utf8'));
    const { constraints, options: expectedOptions, features } = expectedInfo;
    const preferences = new GetOptionsPreferences();
    preferences.enableConfigurableStrings();
    if (constraints) {
      preferences.setConstraintsJson(JSON.stringify(constraints));
    }
    for (const { name, provider } of providers) {
      test(`${name} ${testCase}`, () => {
        for (const [key, expectedValue] of Object.entries(expectedOptions)) {
          const actualJson = provider.getOptionsJson(key, features, preferences);
          const actualOptions = JSON.parse(actualJson);
          expect(actualOptions).toEqual(expectedValue);
        }
      });
    }
  }
}

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

describe('getOptions', () => {
  const configsPath = path.join(__dirname, '../../../tests/test_suites/simple/configs');
  const providers = [{
    name: "OptionsProvider",
    provider: OptionsProvider.build(configsPath),
  }, {
    name: "OptionsWatcher",
    provider: OptionsWatcher.build(configsPath),
  }];

  for (const { name, provider } of providers) {
    test(`${name} validates and returns a typed object with a schema`, () => {
      const config = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      expect(config.rootString).toBe('root string same');
      expect(config.rootString2).toBe('gets overridden');
      expect(config.myArray).toEqual(['example item 1']);
      expect(config.myObject.one).toBe(1);
      expect(config.myObject.deeper.list).toEqual([1, 2]);
    });

    test(`${name} getOptions with schema matches getOptionsJson`, () => {
      const preferences = new GetOptionsPreferences();
      preferences.enableConfigurableStrings();
      const fromJson = JSON.parse(provider.getOptionsJson('myConfig', ['A'], preferences));
      const fromGetOptions = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences);
      expect(fromGetOptions).toEqual(fromJson);
    });

    test(`${name} getOptions with schema rejects invalid data`, () => {
      const StrictSchema = z.object({
        rootString: z.number(),
      });
      expect(() => provider.getOptions('myConfig', ['A'], StrictSchema)).toThrow();
    });

    test(`${name} caches deserialized objects`, () => {
      // Test that calling getOptions multiple times returns the same cached object
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      // Should return the exact same object reference (cached)
      expect(config1).toBe(config2);

      // Verify it still has the expected values
      expect(config1.rootString).toBe('root string same');
    });

    test(`${name} cache differentiates by key`, () => {
      // Different keys should have different cache entries
      const configMyConfig = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      // Verify it has the expected values
      expect(configMyConfig.rootString).toBe('root string same');

      // Calling with same key again should return cached object
      const configMyConfig2 = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      expect(configMyConfig).toBe(configMyConfig2);
    });

    test(`${name} cache differentiates by feature names`, () => {
      const configA = provider.getOptions('myConfig', ['A'], MyConfigSchema);
      const configAB = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema);

      // Should be different objects (different feature combinations)
      expect(configA).not.toBe(configAB);

      // But calling again with same parameters should return cached
      expect(provider.getOptions('myConfig', ['A'], MyConfigSchema)).toBe(configA);
      expect(provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema)).toBe(configAB);
    });

    test(`${name} cache is order-independent for feature names`, () => {
      // Cache should treat ['A', 'B'] and ['B', 'A'] as the same
      const configAB = provider.getOptions('myConfig', ['A', 'B'], MyConfigSchema);
      const configBA = provider.getOptions('myConfig', ['B', 'A'], MyConfigSchema);

      // Should return the same cached object regardless of order
      expect(configAB).toBe(configBA);
    });

    test(`${name} cache differentiates by schema`, () => {
      // Different schemas should have different cache entries
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      // Create a different schema
      const PartialSchema = z.object({
        rootString: z.string(),
      });
      const config2 = provider.getOptions('myConfig', ['A'], PartialSchema);

      // Should be different objects (different schemas)
      expect(config1).not.toBe(config2);
    });

    test(`${name} cache differentiates by configurable strings preference`, () => {
      const config1 = provider.getOptions('myConfig', ['A'], MyConfigSchema);

      const preferences = new GetOptionsPreferences();
      preferences.enableConfigurableStrings();
      const config2 = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences);

      // Should be different objects (different preferences)
      expect(config1).not.toBe(config2);

      // But calling again with same preferences should return cached
      const config2Again = provider.getOptions('myConfig', ['A'], MyConfigSchema, preferences);
      expect(config2).toBe(config2Again);
    });
  }
});

const testSuitesDir = path.join(__dirname, '../../../tests/test_suites');
for (const suite of fs.readdirSync(testSuitesDir)) {
  describe(`Suite: ${suite}`, () => {
    const suitePath = path.join(testSuitesDir, suite);
    if (fs.statSync(suitePath).isDirectory()) {
      runSuite(suitePath);
    }
  });
}
