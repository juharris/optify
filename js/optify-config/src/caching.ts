// Caching utilities for getOptions
import * as nativeBinding from '../index';

/** Options for enabling caching of deserialized objects returned by getOptions.
 * Pass an instance of this class to getOptions to enable caching.
 * Subsequent calls with the same key, feature names, schema, and preferences
 * will return the same cached object without re-parsing.
 */
export class CacheOptions {}

/** Any object with a parse method, compatible with Zod schemas. */
export interface TypeSchema<T> {
  parse(data: unknown): T;
}

// Private cache property names (using symbols for true privacy)
export const CACHE_KEY = Symbol('featuresWithMetadataCache');
export const CACHE_TIME_KEY = Symbol('featuresWithMetadataCacheTime');
export const OPTIONS_CACHE_KEY = Symbol('optionsCache');
export const CACHE_CREATION_TIME_KEY = Symbol('cacheCreationTime');
export const SCHEMA_IDS_KEY = Symbol('schemaIds');
export const SCHEMA_ID_COUNTER_KEY = Symbol('schemaIdCounter');

/** Instance with dynamic properties for caching. */
export interface CacheableInstance {
  _getOptions(key: string, featureNames: string[], preferences?: nativeBinding.GetOptionsPreferences | null): unknown;
  getFilteredFeatures(featureNames: string[], preferences: nativeBinding.GetOptionsPreferences): string[];
  lastModified?(): number;
  [OPTIONS_CACHE_KEY]?: Map<string, unknown>;
  [CACHE_CREATION_TIME_KEY]?: number;
  [SCHEMA_IDS_KEY]?: WeakMap<object, number>;
  [SCHEMA_ID_COUNTER_KEY]?: number;
}

export function getSchemaId(instance: CacheableInstance, schema: object): number {
  // Initialize schema tracking on first use
  if (!instance[SCHEMA_IDS_KEY]) {
    instance[SCHEMA_IDS_KEY] = new WeakMap<object, number>();
    instance[SCHEMA_ID_COUNTER_KEY] = 0;
  }

  const existingId = instance[SCHEMA_IDS_KEY].get(schema);
  if (existingId !== undefined) {
    return existingId;
  }

  const newId = ++instance[SCHEMA_ID_COUNTER_KEY]!;
  instance[SCHEMA_IDS_KEY].set(schema, newId);
  return newId;
}

/**
 * Creates a cache key for getOptions caching.
 * Mirrors Ruby's cache_key: [key, feature_names, are_configurable_strings_enabled, config_class]
 * Note: Constraints are not in the key because features are already filtered.
 */
export function createOptionsCacheKey(
  instance: CacheableInstance,
  key: string,
  featureNames: string[],
  areConfigurableStringsEnabled: boolean,
  schema: object
): string {
  return JSON.stringify([
    key,
    featureNames,
    areConfigurableStringsEnabled,
    getSchemaId(instance, schema)
  ]);
}

/**
 * Resets all caches for the instance.
 * Used by OptionsWatcher when files are modified.
 */
export function resetCaches(instance: any): void {
  instance[OPTIONS_CACHE_KEY] = new Map();
  instance[CACHE_KEY] = undefined;
}

/**
 * Shared implementation of getOptions with optional caching support.
 * Used by both OptionsProvider and OptionsWatcher.
 */
export function getOptionsWithCaching<T>(
  instance: CacheableInstance,
  key: string,
  featureNames: string[],
  schema: TypeSchema<T>,
  preferences?: nativeBinding.GetOptionsPreferences | null,
  cacheOptions?: CacheOptions | null
): T {
  if (cacheOptions) {
    // Check for overrides - caching with overrides is not supported
    if (preferences?.hasOverrides?.()) {
      throw new Error('Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.');
    }

    // Filter features before cache lookup, matching Ruby implementation
    const filterPreferences = preferences || new nativeBinding.GetOptionsPreferences();
    const filteredFeatures = instance.getFilteredFeatures(featureNames, filterPreferences);

    const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

    let cache = instance[OPTIONS_CACHE_KEY];
    if (!cache) {
      cache = new Map();
      instance[OPTIONS_CACHE_KEY] = cache;
    }

    // Use filtered features in cache key
    const cacheKey = createOptionsCacheKey(instance, key, filteredFeatures, areConfigurableStringsEnabled, schema);
    const cached = cache.get(cacheKey);
    if (cached !== undefined) {
      return cached as T;
    }

    // For cache miss, create preferences that skip feature name conversion since features are already filtered
    const cacheMissPreferences = new nativeBinding.GetOptionsPreferences();
    cacheMissPreferences.setSkipFeatureNameConversion(true);
    if (areConfigurableStringsEnabled) {
      cacheMissPreferences.enableConfigurableStrings();
    }

    const result = schema.parse(instance._getOptions(key, filteredFeatures, cacheMissPreferences));
    cache.set(cacheKey, result);
    return result;
  }

  return schema.parse(instance._getOptions(key, featureNames, preferences));
}
