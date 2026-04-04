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

// Augment the native class interfaces to include our new method
declare module '../index' {
  interface OptionsProvider {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Gets options for the specified key and feature names, validated against a schema. */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null): T;
  }

  interface OptionsWatcher {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Gets options for the specified key and feature names, validated against a schema. */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null): T;
  }
}

// Private cache property names (using symbols for true privacy)
const CACHE_KEY = Symbol('featuresWithMetadataCache');
const CACHE_TIME_KEY = Symbol('featuresWithMetadataCacheTime');
const OPTIONS_CACHE_KEY = Symbol('optionsCache');

// Counter for generating unique schema IDs
let schemaIdCounter = 0;

/**
 * Creates a cache key for getOptions caching.
 * Similar to Ruby's cache_key = [key, feature_names, are_configurable_strings_enabled, config_class]
 */
function createOptionsCacheKey(
  key: string,
  featureNames: string[],
  schema: any,
  areConfigurableStringsEnabled: boolean
): string {
  // Sort feature names for consistent cache keys regardless of order
  const sortedFeatures = [...featureNames].sort().join(',');
  // Use schema object reference as a unique identifier
  // Assign a unique numeric ID to each schema object if it doesn't have one
  if (!(schema as any).__cacheId) {
    (schema as any).__cacheId = ++schemaIdCounter;
  }
  const schemaId = (schema as any).__cacheId;
  return `${key}|${sortedFeatures}|${areConfigurableStringsEnabled}|${schemaId}`;
}

/**
 * Gets or initializes the options cache for an instance.
 */
function getOptionsCache(instance: any): Map<string, any> {
  if (!instance[OPTIONS_CACHE_KEY]) {
    instance[OPTIONS_CACHE_KEY] = new Map<string, any>();
  }
  return instance[OPTIONS_CACHE_KEY];
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
(OptionsProvider.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null): any {
  // Determine if configurable strings are enabled
  const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

  // Get or create the cache
  const cache = getOptionsCache(this);

  // Create cache key
  const cacheKey = createOptionsCacheKey(key, featureNames, schema, areConfigurableStringsEnabled);

  // Check cache
  if (cache.has(cacheKey)) {
    return cache.get(cacheKey);
  }

  // Cache miss - get options and parse
  const result = schema.parse(this._getOptions(key, featureNames, preferences));

  // Store in cache
  cache.set(cacheKey, result);

  return result;
};

// Extend OptionsWatcher prototype with extra methods.
export const OptionsWatcher = nativeBinding.OptionsWatcher;
(OptionsWatcher.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedTime = this[CACHE_TIME_KEY];
  const lastModifiedTime = this.lastModified();
  if (cachedTime && lastModifiedTime <= cachedTime) {
    return this[CACHE_KEY];
  }

  this[CACHE_TIME_KEY] = lastModifiedTime;
  return this[CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsWatcher.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null): any {
  // Determine if configurable strings are enabled
  const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

  // Get or create the cache
  const cache = getOptionsCache(this);

  // Create cache key
  const cacheKey = createOptionsCacheKey(key, featureNames, schema, areConfigurableStringsEnabled);

  // Check cache
  if (cache.has(cacheKey)) {
    return cache.get(cacheKey);
  }

  // Cache miss - get options and parse
  const result = schema.parse(this._getOptions(key, featureNames, preferences));

  // Store in cache
  cache.set(cacheKey, result);

  return result;
};