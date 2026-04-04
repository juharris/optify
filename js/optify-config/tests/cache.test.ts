import { describe, expect, test } from '@jest/globals';
import path from 'path';
import { z } from 'zod';
import { CacheOptions, GetOptionsPreferences, OptionsProvider, OptionsWatcher } from '../dist/index';

const configsPath = path.join(__dirname, '../../../tests/test_suites/simple/configs');

const LooseConfigSchema = z.object({}).passthrough();

const buildProviders = () => ([
  { name: 'OptionsProvider', provider: OptionsProvider.build(configsPath) },
  { name: 'OptionsWatcher', provider: OptionsWatcher.build(configsPath) },
]);

describe('getOptions cache', () => {
  test('works without caching when cacheOptions is omitted', () => {
    for (const { provider } of buildProviders()) {
      const preferences = new GetOptionsPreferences();
      preferences.enableConfigurableStrings();

      const config = provider.getOptions('myConfig', ['A'], LooseConfigSchema, null, preferences);
      expect(config.rootString).toBe('root string same');
    }
  });

  test('caches parsed objects per canonical feature set and schema', () => {
    for (const { name, provider } of buildProviders()) {
      const cacheOptions = new CacheOptions();

      const configA = provider.getOptions('myConfig', ['A'], LooseConfigSchema, cacheOptions);
      const configB = provider.getOptions('myConfig', ['B'], LooseConfigSchema, cacheOptions);
      expect(configA).not.toBe(configB);

      const configB2 = provider.getOptions('myConfig', ['b'], LooseConfigSchema, cacheOptions);
      expect(configB2).toBe(configB);

      const configBAlias = provider.getOptions('myConfig', ['featUre_B/iNITial'], LooseConfigSchema, cacheOptions);
      expect(configBAlias).toBe(configB);

      const configAB = provider.getOptions('myConfig', ['A', 'B'], LooseConfigSchema, cacheOptions);
      const configBA = provider.getOptions('myConfig', ['B', 'A'], LooseConfigSchema, cacheOptions);
      expect(configAB).not.toBe(configBA);

      const configABAlias = provider.getOptions('myConfig', ['A', 'featUre_B/iNITial'], LooseConfigSchema, cacheOptions);
      expect(configABAlias).toBe(configAB);

    }
  });

  test('uses configurable strings preference in the cache key', () => {
    for (const { name, provider } of buildProviders()) {
      const cacheOptions = new CacheOptions();

      const defaultConfig = provider.getOptions('myConfig', ['A'], LooseConfigSchema, cacheOptions);

      const preferences = new GetOptionsPreferences();
      preferences.enableConfigurableStrings();

      const configurableConfig = provider.getOptions('myConfig', ['A'], LooseConfigSchema, cacheOptions, preferences);
      expect(configurableConfig).not.toBe(defaultConfig);

      const configurableConfigAlias = provider.getOptions('myConfig', ['a'], LooseConfigSchema, cacheOptions, preferences);
      expect(configurableConfigAlias).toBe(configurableConfig);

    }
  });
});
