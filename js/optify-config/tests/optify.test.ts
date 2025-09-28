import { describe, expect, test } from '@jest/globals';
import fs from 'fs';
import path from 'path';
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

const testSuitesDir = path.join(__dirname, '../../../tests/test_suites');
for (const suite of fs.readdirSync(testSuitesDir)) {
  describe(`Suite: ${suite}`, () => {
    const suitePath = path.join(testSuitesDir, suite);
    if (fs.statSync(suitePath).isDirectory()) {
      runSuite(suitePath);
    }
  });
}
