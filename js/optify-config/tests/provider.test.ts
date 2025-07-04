import { describe, expect, test } from '@jest/globals';
import fs from 'fs';
import path from 'path';
import { OptionsProvider } from "../index";

const configsPath = path.join(__dirname, '../../../tests/test_suites/simple/configs')
const expectationsPath = path.join(configsPath, '../expectations')
const getProvider = () => {
  return OptionsProvider.buildFromDirectories([configsPath])
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

  test('invalid file', () => {
    const configDir = path.relative(__dirname, path.join(__dirname, '../../rust/optify/tests/invalid_file'))
    const filePath = path.join(configDir, 'invalid.yaml')
    expect(() => OptionsProvider.build(configDir))
      .toThrow(`Error loading file '${filePath}': simple key expected at byte 31 line 4 column 1 in ${filePath}`)
  })
})