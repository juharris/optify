import { describe, expect, test } from '@jest/globals';
import fs from 'fs';
import path from 'path';
import { OptionsProviderBuilder } from "../index";

const configsPath = path.join(__dirname, '../../../tests/test_suites/simple/configs')
const expectationsPath = path.join(configsPath, '../expectations')
const getProvider = () => {
  const builder = new OptionsProviderBuilder()
  builder.addDirectory(configsPath)
  return builder.build()
};

describe('Provider', () => {
  const provider = getProvider()
  test('get_all_options_json feature_A', () => {
    const options = JSON.parse(provider.getAllOptionsJson(['feature_A']))
    const expectedOptions = JSON.parse(fs.readFileSync(path.join(configsPath, 'feature_A.json'), 'utf8'))['options']
    expect(options).toEqual(expectedOptions)
  })

  test('get_all_options_json A and B', () => {
    const options = JSON.parse(provider.getAllOptionsJson(['A', 'B']))
    const expectedOptions = JSON.parse(fs.readFileSync(path.join(expectationsPath, 'aliases.json'), 'utf8'))['options']
    expect(options).toEqual(expectedOptions)
  })
})