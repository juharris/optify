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

// Options for enabling getOptions caching. The presence of an instance enables caching.
export class CacheOptions { }

type PreferencesState = {
  skipFeatureNameConversion: boolean;
  constraintsJson: string | null;
  hasOverrides: boolean;
  configurableStringsEnabled: boolean;
};

// Augment the native class interfaces to include our new method
declare module '../index' {
  interface OptionsProvider {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Returns canonical feature names after applying constraints and conversions. */
    getFilteredFeatures(featureNames: Array<string>, preferences?: GetOptionsPreferences | null): Array<string>;
    /** Gets options for the specified key and feature names, validated against a schema. */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null): T;
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, cacheOptions: CacheOptions | null, preferences?: GetOptionsPreferences | null): T;
  }

  interface OptionsWatcher {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /** Returns canonical feature names after applying constraints and conversions. */
    getFilteredFeatures(featureNames: Array<string>, preferences?: GetOptionsPreferences | null): Array<string>;
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
const PREFERENCES_STATE_KEY = Symbol('preferencesState');
const DEFAULT_FILTER_PREFERENCES = new nativeBinding.GetOptionsPreferences();

const preferencesPrototype = (nativeBinding.GetOptionsPreferences as any).prototype;
const originalSetConstraintsJson = preferencesPrototype.setConstraintsJson;
preferencesPrototype.setConstraintsJson = function (this: any, constraintsJson?: string | null) {
  const state = ensurePreferencesState(this);
  state.constraintsJson = constraintsJson ?? null;
  return originalSetConstraintsJson.call(this, constraintsJson);
};

const originalSetOverridesJson = preferencesPrototype.setOverridesJson;
preferencesPrototype.setOverridesJson = function (this: any, overridesJson?: string | null) {
  const state = ensurePreferencesState(this);
  state.hasOverrides = !!overridesJson;
  return originalSetOverridesJson.call(this, overridesJson);
};

const originalSetSkipFeatureNameConversion = preferencesPrototype.setSkipFeatureNameConversion;
preferencesPrototype.setSkipFeatureNameConversion = function (this: any, skip: boolean) {
  const state = ensurePreferencesState(this);
  state.skipFeatureNameConversion = skip;
  return originalSetSkipFeatureNameConversion.call(this, skip);
};

const originalEnableConfigurableStrings = preferencesPrototype.enableConfigurableStrings;
preferencesPrototype.enableConfigurableStrings = function (this: any) {
  const state = ensurePreferencesState(this);
  state.configurableStringsEnabled = true;
  return originalEnableConfigurableStrings.call(this);
};

const originalDisableConfigurableStrings = preferencesPrototype.disableConfigurableStrings;
preferencesPrototype.disableConfigurableStrings = function (this: any) {
  const state = ensurePreferencesState(this);
  state.configurableStringsEnabled = false;
  return originalDisableConfigurableStrings.call(this);
};

function ensurePreferencesState(preferences: any): PreferencesState {
  if (!preferences[PREFERENCES_STATE_KEY]) {
    preferences[PREFERENCES_STATE_KEY] = {
      constraintsJson: null,
      hasOverrides: false,
      skipFeatureNameConversion: false,
      configurableStringsEnabled: preferences.areConfigurableStringsEnabled
        ? preferences.areConfigurableStringsEnabled()
        : false
    } as PreferencesState;
  }

  return preferences[PREFERENCES_STATE_KEY] as PreferencesState;
}

function splitCacheOptions(
  cacheOptionsOrPreferences?: CacheOptions | nativeBinding.GetOptionsPreferences | null,
  preferences?: nativeBinding.GetOptionsPreferences | null
): { cacheOptions: CacheOptions | null; preferences: nativeBinding.GetOptionsPreferences | null } {
  if (cacheOptionsOrPreferences instanceof CacheOptions || preferences !== undefined) {
    return {
      cacheOptions: (cacheOptionsOrPreferences as CacheOptions) ?? null,
      preferences: preferences ?? null
    };
  }

  return {
    cacheOptions: null,
    preferences: (cacheOptionsOrPreferences as nativeBinding.GetOptionsPreferences | null) ?? null
  };
}

function normalizeFeatureNamesForCache(
  instance: any,
  featureNames: string[],
  preferences: nativeBinding.GetOptionsPreferences | null
): { featureNames: string[]; configurableStringsEnabled: boolean; preferencesState: PreferencesState | null } {
  const preferencesState = preferences ? ensurePreferencesState(preferences) : null;
  const shouldFilter =
    !preferences || !preferencesState?.skipFeatureNameConversion || !!preferencesState?.constraintsJson;
  const normalizedFeatures = shouldFilter
    ? instance.getFilteredFeatures(featureNames, preferences ?? DEFAULT_FILTER_PREFERENCES)
    : featureNames;
  const canonicalFeatures = normalizedFeatures.slice();
  const configurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

  return {
    featureNames: canonicalFeatures,
    configurableStringsEnabled,
    preferencesState
  };
}

function buildCachePreferences(configurableStringsEnabled: boolean): nativeBinding.GetOptionsPreferences {
  const cachePreferences = new nativeBinding.GetOptionsPreferences();
  cachePreferences.setSkipFeatureNameConversion(true);
  if (configurableStringsEnabled) {
    cachePreferences.enableConfigurableStrings();
  }
  return cachePreferences;
}

function buildCacheKey(key: string, featureNames: string[], configurableStringsEnabled: boolean): string {
  return JSON.stringify([key, featureNames, configurableStringsEnabled]);
}

function ensureOptionsCache(instance: any, schema: any): Map<string, any> {
  if (!instance[OPTIONS_CACHE_KEY]) {
    instance[OPTIONS_CACHE_KEY] = new Map();
  }

  const cacheBySchema = instance[OPTIONS_CACHE_KEY].get(schema);
  if (cacheBySchema) {
    return cacheBySchema;
  }

  const newCache = new Map();
  instance[OPTIONS_CACHE_KEY].set(schema, newCache);
  return newCache;
}

function maybeResetWatcherOptionsCache(instance: any) {
  if (typeof instance.lastModified !== 'function') {
    return;
  }

  const cachedTime = instance[OPTIONS_CACHE_TIME_KEY];
  const lastModifiedTime = instance.lastModified();
  if (!cachedTime || lastModifiedTime > cachedTime) {
    instance[OPTIONS_CACHE_KEY] = new Map();
    instance[OPTIONS_CACHE_TIME_KEY] = lastModifiedTime;
  }
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
  const { cacheOptions, preferences: parsedPreferences } = splitCacheOptions(cacheOptionsOrPreferences, preferences);
  if (!cacheOptions) {
    return schema.parse(this._getOptions(key, featureNames, parsedPreferences));
  }

  const { featureNames: canonicalFeatures, configurableStringsEnabled, preferencesState } = normalizeFeatureNamesForCache(
    this,
    featureNames,
    parsedPreferences
  );

  if (preferencesState?.hasOverrides) {
    throw new Error('Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.');
  }

  const cacheKey = buildCacheKey(key, canonicalFeatures, configurableStringsEnabled);
  const cacheBySchema = ensureOptionsCache(this, schema);
  if (cacheBySchema.has(cacheKey)) {
    return cacheBySchema.get(cacheKey);
  }

  const cachePreferences = buildCachePreferences(configurableStringsEnabled);
  const parsed = schema.parse(this._getOptions(key, canonicalFeatures, cachePreferences));
  cacheBySchema.set(cacheKey, parsed);
  return parsed;
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
  const { cacheOptions, preferences: parsedPreferences } = splitCacheOptions(cacheOptionsOrPreferences, preferences);
  if (!cacheOptions) {
    return schema.parse(this._getOptions(key, featureNames, parsedPreferences));
  }

  maybeResetWatcherOptionsCache(this);
  const { featureNames: canonicalFeatures, configurableStringsEnabled, preferencesState } = normalizeFeatureNamesForCache(
    this,
    featureNames,
    parsedPreferences
  );

  if (preferencesState?.hasOverrides) {
    throw new Error('Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.');
  }

  const cacheKey = buildCacheKey(key, canonicalFeatures, configurableStringsEnabled);
  const cacheBySchema = ensureOptionsCache(this, schema);
  if (cacheBySchema.has(cacheKey)) {
    return cacheBySchema.get(cacheKey);
  }

  const cachePreferences = buildCachePreferences(configurableStringsEnabled);
  const parsed = schema.parse(this._getOptions(key, canonicalFeatures, cachePreferences));
  cacheBySchema.set(cacheKey, parsed);
  return parsed;
};
