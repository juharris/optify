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
