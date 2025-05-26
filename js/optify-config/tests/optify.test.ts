import { describe, expect, test } from '@jest/globals';
import { OptionsProviderBuilder } from "../index";
import path from 'path';
import fs from 'fs';

const runSuite = (suitePath: string) => {
  const builder = new OptionsProviderBuilder();
  builder.addDirectory(path.join(suitePath, 'configs'));
  const provider = builder.build();
  const expectationsPath = path.join(suitePath, 'expectations');
  for (const testCase of fs.readdirSync(expectationsPath)) {
    const expectationPath = path.join(expectationsPath, testCase);
    const expectedInfo = JSON.parse(fs.readFileSync(expectationPath, 'utf8'));
    const { options: expectedOptions, features } = expectedInfo;
    for (const [key, expectedValue] of Object.entries(expectedOptions)) {
      const actualJson = provider.getOptionsJson(key, features);
      const actualOptions = JSON.parse(actualJson);
      expect(actualOptions).toEqual(expectedValue);
    }
  }
};

describe('Suites', () => {
  const testSuitesDir = path.join(__dirname, '../../../tests/test_suites');
  for (const suite of fs.readdirSync(testSuitesDir)) {
    const suitePath = path.join(testSuitesDir, suite);
    if (fs.statSync(suitePath).isDirectory()) {
      test(`Suite: ${suite}`, () => {
        runSuite(suitePath);
      });
    }
  }
});
