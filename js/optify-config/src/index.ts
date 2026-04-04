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

// WeakMap to store schema IDs without mutating the schema objects.
// JavaScript runs on a single event-loop thread; worker threads have isolated heaps,
// so this counter is safe in all standard Node.js usage.
const schemaIds = new WeakMap<object, number>();
let schemaIdCounter = 0;

function getSchemaId(schema: any): number {
  let id = schemaIds.get(schema);
  if (id === undefined) {
    id = ++schemaIdCounter;
    schemaIds.set(schema, id);
  }
  return id;
}

/**
 * Creates a cache key for getOptions caching.
 * Mirrors Ruby's cache_key: [key, feature_names, are_configurable_strings_enabled, config_class]
 */
function createOptionsCacheKey(
  key: string,
  featureNames: string[],
  areConfigurableStringsEnabled: boolean,
  schema: any
): string {
  return JSON.stringify([
    key,
    featureNames,
    areConfigurableStringsEnabled,
    getSchemaId(schema)
  ]);
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
  if (cacheOptions) {
    const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

    let cache: Map<string, any> = this[OPTIONS_CACHE_KEY];
    if (!cache) {
      cache = new Map();
      this[OPTIONS_CACHE_KEY] = cache;
    }

    const cacheKey = createOptionsCacheKey(key, featureNames, areConfigurableStringsEnabled, schema);
    const cached = cache.get(cacheKey);
    if (cached !== undefined) {
      return cached;
    }

    const result = schema.parse(this._getOptions(key, featureNames, preferences));
    cache.set(cacheKey, result);
    return result;
  }

  return schema.parse(this._getOptions(key, featureNames, preferences));
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
(OptionsWatcher.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null, cacheOptions?: CacheOptions | null): any {
  if (cacheOptions) {
    const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

    let cache: Map<string, any> = this[OPTIONS_CACHE_KEY];
    if (!cache) {
      cache = new Map();
      this[OPTIONS_CACHE_KEY] = cache;
    }

    const cacheKey = createOptionsCacheKey(key, featureNames, areConfigurableStringsEnabled, schema);
    const cached = cache.get(cacheKey);
    if (cached !== undefined) {
      return cached;
    }

    const result = schema.parse(this._getOptions(key, featureNames, preferences));
    cache.set(cacheKey, result);
    return result;
  }

  return schema.parse(this._getOptions(key, featureNames, preferences));
};
