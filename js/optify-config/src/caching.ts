// Caching utilities for getOptions

import { LRUCache } from 'lru-cache';
import * as nativeBinding from '../index';
import { TypeSchema } from './types';

/**
 * Options for enabling caching of deserialized objects returned by getOptions.
 * Pass an instance of this class to getOptions to enable caching.
 * Subsequent calls with the same key, feature names, schema, and preferences
 * will return the same cached object without re-parsing.
 */
export class CacheOptions { }

/**
 * Configuration options for cache initialization.
 * Used when building providers/watchers to configure cache behavior.
 */
export class CacheInitOptions {
  /**
   * The maximum number of entries to keep in the cache.
   * When the cache is full, the least recently used entry will be evicted.
   * If not set, the cache size is unlimited.
   */
  readonly maxSize?: number;

  constructor(maxSize?: number) {
    this.maxSize = maxSize;
  }
}

// Private cache property names (using symbols for true privacy)
export const FEATURES_WITH_METADATA_CACHE_KEY = Symbol('featuresWithMetadataCache');
export const FEATURES_WITH_METADATA_CACHE_TIME_KEY = Symbol('featuresWithMetadataCacheTime');
const OPTIONS_CACHE_KEY = Symbol('optionsCache');
export const CACHE_CREATION_TIME_KEY = Symbol('cacheCreationTime');
const CACHE_INIT_OPTIONS_KEY = Symbol('cacheInitOptions');
const SCHEMA_IDS_KEY = Symbol('schemaIds');
const SCHEMA_ID_COUNTER_KEY = Symbol('schemaIdCounter');

type OptionsCache = Map<string, NonNullable<unknown>> | LRUCache<string, NonNullable<unknown>>;

/** Instance with dynamic properties for caching. */
export interface CacheableInstance {
  _getOptions(key: string, featureNames: string[], preferences?: nativeBinding.GetOptionsPreferences | null): unknown;
  getFilteredFeatures(featureNames: string[], preferences: nativeBinding.GetOptionsPreferences): string[];
  lastModified?(): number;
  [FEATURES_WITH_METADATA_CACHE_KEY]?: Record<string, nativeBinding.OptionsMetadata>;
  [OPTIONS_CACHE_KEY]?: OptionsCache;
  [CACHE_INIT_OPTIONS_KEY]?: CacheInitOptions | null;
  [SCHEMA_ID_COUNTER_KEY]?: number;
  [SCHEMA_IDS_KEY]?: WeakMap<object, number>;
}

function getSchemaId(instance: CacheableInstance, schema: object): number {
  let schemaIds = instance[SCHEMA_IDS_KEY];
  if (!schemaIds) {
    schemaIds = new WeakMap<object, number>();
    instance[SCHEMA_IDS_KEY] = schemaIds;
    instance[SCHEMA_ID_COUNTER_KEY] = 0;
  }

  const existingId = schemaIds.get(schema);
  if (existingId !== undefined) {
    return existingId;
  }

  const newId = ++instance[SCHEMA_ID_COUNTER_KEY]!;
  schemaIds.set(schema, newId);
  return newId;
}

/**
 * Creates a cache key for getOptions caching.
 * Note: Constraints are not in the key because features are already filtered.
 */
function createOptionsCacheKey(
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

function createOptionsCache(cacheInitOptions?: CacheInitOptions | null): OptionsCache {
  if (cacheInitOptions?.maxSize !== undefined) {
    return new LRUCache<string, NonNullable<unknown>>({ max: cacheInitOptions.maxSize });
  }
  return new Map<string, NonNullable<unknown>>();
}

/**
 * Resets all caches for the instance, preserving the configured max size.
 * Used by OptionsWatcher when files are modified.
 */
export function resetCaches(instance: CacheableInstance): void {
  instance[FEATURES_WITH_METADATA_CACHE_KEY] = undefined;
  instance[OPTIONS_CACHE_KEY] = createOptionsCache(instance[CACHE_INIT_OPTIONS_KEY]);
}

/**
 * Eagerly initializes the cache for the instance.
 * Like Ruby's `init` method, this should be called before `getOptions` to configure caching.
 * @param instance The cacheable instance to initialize.
 * @param cacheInitOptions Optional cache initialization options to configure cache behavior.
 */
export function initCache(instance: CacheableInstance, cacheInitOptions?: CacheInitOptions | null): void {
  instance[CACHE_INIT_OPTIONS_KEY] = cacheInitOptions;
  instance[OPTIONS_CACHE_KEY] = createOptionsCache(cacheInitOptions);
}

/**
 * Shared implementation of getOptions with optional caching support.
 * Used by both `OptionsProvider` and `OptionsWatcher`.
 * Caching is enabled when the instance has been initialized via `init`.
 */
export function getOptionsWithCaching<T>(
  instance: CacheableInstance,
  key: string,
  featureNames: string[],
  schema: TypeSchema<T>,
  preferences?: nativeBinding.GetOptionsPreferences | null,
  cacheOptions?: CacheOptions | null
): T {
  const cache = instance[OPTIONS_CACHE_KEY];
  if (cacheOptions && cache) {
    if (preferences?.hasOverrides?.()) {
      throw new Error('Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.');
    }

    const filterPreferences = preferences || new nativeBinding.GetOptionsPreferences();
    const filteredFeatures = instance.getFilteredFeatures(featureNames, filterPreferences);

    const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

    const cacheKey = createOptionsCacheKey(instance, key, filteredFeatures, areConfigurableStringsEnabled, schema);
    const cachedResult = cache.get(cacheKey);
    if (cachedResult !== undefined) {
      return cachedResult as T;
    }

    // For cache miss, create preferences that skip feature name conversion since features are already filtered.
    const cacheMissPreferences = new nativeBinding.GetOptionsPreferences();
    cacheMissPreferences.setSkipFeatureNameConversion(true);
    if (areConfigurableStringsEnabled) {
      cacheMissPreferences.enableConfigurableStrings();
    }

    const result = schema.parse(instance._getOptions(key, filteredFeatures, cacheMissPreferences));
    cache.set(cacheKey, result as NonNullable<unknown>);
    return result;
  }

  return schema.parse(instance._getOptions(key, featureNames, preferences));
}
