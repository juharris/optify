// TypeScript wrapper for the native bindings with additional optimizations and definitions.

// Import the generated native bindings.
import * as nativeBinding from '../index';

// Re-export types that don't need modifications.
export {
  OptionsMetadata,
  GetOptionsPreferences,
  OptionsProviderBuilder,
  OptionsWatcherListenerEvent,
  OptionsWatcherBuilder,
  WatcherOptions
} from '../index';

// Re-export the native classes directly
export type OptionsProvider = nativeBinding.OptionsProvider;
export type OptionsWatcher = nativeBinding.OptionsWatcher;

/** Any object with a parse method, compatible with Zod schemas. */
export interface TypeSchema<T> {
  parse(data: unknown): T;
}

/** Options for enabling caching of deserialized objects returned by getOptions.
 * Pass an instance of this class to getOptions to enable caching.
 * Subsequent calls with the same key, feature names, schema, and preferences
 * will return the same cached object without re-parsing.
 */
export class CacheOptions {}

// Augment the native class interfaces to include our new method
declare module '../index' {
  interface OptionsProvider {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Gets options for the specified key and feature names, validated against a schema. */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions | null): T;
  }

  interface OptionsWatcher {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Gets options for the specified key and feature names, validated against a schema. */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions | null): T;
  }
}

// Private cache property names (using symbols for true privacy)
const CACHE_KEY = Symbol('featuresWithMetadataCache');
const CACHE_TIME_KEY = Symbol('featuresWithMetadataCacheTime');
const OPTIONS_CACHE_KEY = Symbol('optionsCache');
const CACHE_CREATION_TIME_KEY = Symbol('cacheCreationTime');
const SCHEMA_IDS_KEY = Symbol('schemaIds');
const SCHEMA_ID_COUNTER_KEY = Symbol('schemaIdCounter');

/** Instance with dynamic properties for caching. */
interface CacheableInstance {
  _getOptions(key: string, featureNames: string[], preferences?: nativeBinding.GetOptionsPreferences | null): unknown;
  getFilteredFeatures(featureNames: string[], preferences: nativeBinding.GetOptionsPreferences): string[];
  lastModified?(): number;
  [OPTIONS_CACHE_KEY]?: Map<string, unknown>;
  [CACHE_CREATION_TIME_KEY]?: number;
  [SCHEMA_IDS_KEY]?: WeakMap<object, number>;
  [SCHEMA_ID_COUNTER_KEY]?: number;
}

function getSchemaId(instance: CacheableInstance, schema: object): number {
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

/**
 * Resets all caches for the instance.
 * Used by OptionsWatcher when files are modified.
 */
function resetCaches(instance: any): void {
  instance[OPTIONS_CACHE_KEY] = new Map();
  instance[CACHE_KEY] = undefined;
}

/**
 * Shared implementation of getOptions with optional caching support.
 * Used by both OptionsProvider and OptionsWatcher.
 */
function getOptionsWithCaching<T>(
  instance: CacheableInstance,
  key: string,
  featureNames: string[],
  schema: TypeSchema<T>,
  preferences?: nativeBinding.GetOptionsPreferences | null,
  cacheOptions?: CacheOptions | null
): T {
  if (cacheOptions) {
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


// Extend OptionsProvider prototype with extra methods.
export const OptionsProvider = nativeBinding.OptionsProvider;
(OptionsProvider.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedResult = this[CACHE_KEY];
  if (cachedResult) {
    return cachedResult;
  }

  return this[CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsProvider.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null, cacheOptions?: CacheOptions | null): any {
  return getOptionsWithCaching(this, key, featureNames, schema, preferences, cacheOptions);
};

// Extend OptionsWatcher prototype with extra methods.
export const OptionsWatcher = nativeBinding.OptionsWatcher;
(OptionsWatcher.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedTime = this[CACHE_TIME_KEY];
  const lastModifiedTime = this.lastModified();
  if (cachedTime && lastModifiedTime <= cachedTime) {
    return this[CACHE_KEY];
  }

  // Cache is stale, reset all caches
  resetCaches(this);
  this[CACHE_TIME_KEY] = lastModifiedTime;
  return this[CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsWatcher.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null, cacheOptions?: CacheOptions | null): any {
  // Check cache validity for watcher - reset if files have been modified
  if (cacheOptions) {
    const lastModifiedTime = this.lastModified();
    const cacheCreationTime = this[CACHE_CREATION_TIME_KEY];

    if (!cacheCreationTime || lastModifiedTime > cacheCreationTime) {
      // Cache is stale, reset all caches
      resetCaches(this);
      this[CACHE_CREATION_TIME_KEY] = lastModifiedTime;
    }
  }

  return getOptionsWithCaching(this, key, featureNames, schema, preferences, cacheOptions);
};
