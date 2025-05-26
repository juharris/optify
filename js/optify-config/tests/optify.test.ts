import {describe, expect, test} from '@jest/globals';
import { OptionsProviderBuilder } from "../index";
import path from 'path';
import fs from 'fs';

const runSuite = (suitePath: string) => {
  console.log(`Running suite at ${suitePath}`);
  const builder = new OptionsProviderBuilder();
  builder.addDirectory(path.join(suitePath, 'configs'));
  const provider = builder.build();
  const expectationsPath = path.join(suitePath, 'expectations');
  for (const testCase of fs.readdirSync(expectationsPath)) {
    const expectationPath = path.join(expectationsPath, testCase);
    const expectedInfo = JSON.parse(fs.readFileSync(expectationPath, 'utf8'));
    const {expectedOptions, features} = expectedInfo;
    // TODO
  }
};

describe('Suites', () => {
  test('should pass suites', () => {
    const testSuitesDir = path.join(__dirname, '../../../tests/test_suites');
    for (const suite of fs.readdirSync(testSuitesDir)) {
      const suitePath = path.join(testSuitesDir, suite);
      if (fs.statSync(suitePath).isDirectory()) {
        runSuite(suitePath);
      }
    }
  });
});
