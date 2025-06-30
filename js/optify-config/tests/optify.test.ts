import { describe, expect, test } from '@jest/globals';
import fs from 'fs';
import path from 'path';
import { OptionsProvider } from "../index";

const runSuite = (suitePath: string) => {
  const provider = OptionsProvider.build(path.join(suitePath, 'configs'));
  const expectationsPath = path.join(suitePath, 'expectations');
  for (const testCase of fs.readdirSync(expectationsPath)) {
    const expectationPath = path.join(expectationsPath, testCase);
    test(`${testCase}`, () => {
      const expectedInfo = JSON.parse(fs.readFileSync(expectationPath, 'utf8'));
      const { options: expectedOptions, features } = expectedInfo;
      for (const [key, expectedValue] of Object.entries(expectedOptions)) {
        const actualJson = provider.getOptionsJson(key, features);
        const actualOptions = JSON.parse(actualJson);
        expect(actualOptions).toEqual(expectedValue);
      }
    });
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
