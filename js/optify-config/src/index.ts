// TypeScript wrapper for the native bindings with additional optimizations and definitions.

// Import the generated native bindings.
import * as nativeBinding from '../index';
import { randomUUID } from 'crypto';

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

// Options for enabling getOptions caching. The presence of an instance enables caching.
export class CacheOptions { }

// Augment the native class interfaces to include our new method
declare module '../index' {
  interface OptionsProvider {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Gets options for the specified key and feature names, validated against a schema. */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null): T;
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, cacheOptions: CacheOptions | null, preferences?: GetOptionsPreferences | null): T;
  }

  interface OptionsWatcher {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Gets options for the specified key and feature names, validated against a schema. */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null): T;
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, cacheOptions: CacheOptions | null, preferences?: GetOptionsPreferences | null): T;
  }
}

// Private cache property names (using symbols for true privacy)
const FEATURES_CACHE_KEY = Symbol('featuresWithMetadataCache');
const FEATURES_CACHE_TIME_KEY = Symbol('featuresWithMetadataCacheTime');
const OPTIONS_CACHE_KEY = Symbol('optionsCache');
const OPTIONS_CACHE_TIME_KEY = Symbol('optionsCacheTime');

// Assign unique IDs to schema objects for use in flat cache keys
const _schemaIds = new WeakMap<object, string>();

function getSchemaId(schema: object): string {
  let id = _schemaIds.get(schema);
  if (id === undefined) {
    id = randomUUID();
    _schemaIds.set(schema, id);
  }
  return id;
}

function buildCacheKey(key: string, featureNames: string[], configurableStringsEnabled: boolean, schemaId: string): string {
  return JSON.stringify([key, featureNames, configurableStringsEnabled, schemaId]);
}

function maybeResetWatcherOptionsCache(instance: any) {
  if (typeof instance.lastModified !== 'function') {
    return;
  }

  const cachedTime = instance[OPTIONS_CACHE_TIME_KEY];
  const lastModifiedTime = instance.lastModified();
  if (!cachedTime || lastModifiedTime > cachedTime) {
    instance[OPTIONS_CACHE_KEY] = new Map<string, any>();
    instance[OPTIONS_CACHE_TIME_KEY] = lastModifiedTime;
  }
}

function getOptionsWithCache(
  instance: any,
  key: string,
  featureNames: string[],
  schema: any,
  preferences: nativeBinding.GetOptionsPreferences | null | undefined
): any {
  const configurableStringsEnabled = preferences?.areConfigurableStringsEnabled() ?? false;
  const canonicalFeatures: string[] = instance.getFilteredFeatures(featureNames, preferences ?? null);
  const schemaId = getSchemaId(schema);
  const cacheKey = buildCacheKey(key, canonicalFeatures, configurableStringsEnabled, schemaId);

  if (!instance[OPTIONS_CACHE_KEY]) {
    instance[OPTIONS_CACHE_KEY] = new Map<string, any>();
  }
  const cache: Map<string, any> = instance[OPTIONS_CACHE_KEY];

  if (cache.has(cacheKey)) {
    return cache.get(cacheKey);
  }

  const cachePreferences = new nativeBinding.GetOptionsPreferences();
  cachePreferences.setSkipFeatureNameConversion(true);
  if (configurableStringsEnabled) {
    cachePreferences.enableConfigurableStrings();
  }

  const parsed = schema.parse(instance._getOptions(key, canonicalFeatures, cachePreferences));
  cache.set(cacheKey, parsed);
  return parsed;
}

// Extend OptionsProvider prototype with extra methods.
export const OptionsProvider = nativeBinding.OptionsProvider;
(OptionsProvider.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedResult = this[FEATURES_CACHE_KEY];
  if (cachedResult) {
    return cachedResult;
  }

  return this[FEATURES_CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsProvider.prototype as any).getOptions = function (
  this: any,
  key: string,
  featureNames: string[],
  schema: any,
  cacheOptionsOrPreferences?: CacheOptions | nativeBinding.GetOptionsPreferences | null,
  preferences?: nativeBinding.GetOptionsPreferences | null
): any {
  const legacyPreferences = cacheOptionsOrPreferences instanceof nativeBinding.GetOptionsPreferences ? cacheOptionsOrPreferences : null;
  const cacheOptions = legacyPreferences ? null : cacheOptionsOrPreferences;
  const resolvedPreferences = legacyPreferences ?? preferences ?? null;

  if (!cacheOptions) {
    return schema.parse(this._getOptions(key, featureNames, resolvedPreferences));
  }
  return getOptionsWithCache(this, key, featureNames, schema, resolvedPreferences);
};

// Extend OptionsWatcher prototype with extra methods.
export const OptionsWatcher = nativeBinding.OptionsWatcher;
(OptionsWatcher.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedTime = this[FEATURES_CACHE_TIME_KEY];
  const lastModifiedTime = this.lastModified();
  if (cachedTime && lastModifiedTime <= cachedTime) {
    return this[FEATURES_CACHE_KEY];
  }

  this[FEATURES_CACHE_TIME_KEY] = lastModifiedTime;
  return this[FEATURES_CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsWatcher.prototype as any).getOptions = function (
  this: any,
  key: string,
  featureNames: string[],
  schema: any,
  cacheOptionsOrPreferences?: CacheOptions | nativeBinding.GetOptionsPreferences | null,
  preferences?: nativeBinding.GetOptionsPreferences | null
): any {
  const legacyPreferences = cacheOptionsOrPreferences instanceof nativeBinding.GetOptionsPreferences ? cacheOptionsOrPreferences : null;
  const cacheOptions = legacyPreferences ? null : cacheOptionsOrPreferences;
  const resolvedPreferences = legacyPreferences ?? preferences ?? null;

  if (!cacheOptions) {
    return schema.parse(this._getOptions(key, featureNames, resolvedPreferences));
  }

  maybeResetWatcherOptionsCache(this);
  return getOptionsWithCache(this, key, featureNames, schema, resolvedPreferences);
};
