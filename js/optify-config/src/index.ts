// TypeScript wrapper for the native bindings with additional optimizations and definitions.

// Import the generated native bindings.
import * as nativeBinding from '../index';

import {
  CACHE_CREATION_TIME_KEY,
  CacheInitOptions,
  CacheOptions,
  FEATURES_WITH_METADATA_CACHE_KEY,
  FEATURES_WITH_METADATA_CACHE_TIME_KEY,
  getOptionsWithCaching,
  initCache,
  resetCaches
} from './caching';
import { TypeSchema } from './types';

// Re-export types that don't need modifications.
export {
  GetOptionsPreferences,
  OptionsMetadata,
  OptionsProviderBuilder,
  OptionsWatcherBuilder,
  OptionsWatcherListenerEvent,
  WatcherOptions
} from '../index';

// Re-export the native classes directly
export type OptionsProvider = nativeBinding.OptionsProvider;
export type OptionsWatcher = nativeBinding.OptionsWatcher;

export { CacheInitOptions, CacheOptions } from './caching';
export type { TypeSchema } from './types';

// Augment the native class interfaces to include our new method
declare module '../index' {
  interface OptionsProvider {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /**
     * Eagerly initializes the cache. Call this before `getOptions` to enable caching.
     * @param cacheInitOptions Optional cache initialization options to configure cache behavior.
     * @returns `this` for chaining.
     */
    init(cacheInitOptions?: CacheInitOptions | null): OptionsProvider;
    /**
     * Gets options for the specified key and feature names, validated against a schema.
     * @param cacheOptions Optional cache options to enable caching of the deserialized result. The cache must be initialized via `init` first.
     */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions): T;
  }

  interface OptionsWatcher {
    /** Returns a map of all the canonical feature names to their metadata. */
    featuresWithMetadata(): Record<string, OptionsMetadata>;
    /**
     * Eagerly initializes the cache. Call this before `getOptions` to enable caching.
     * @param cacheInitOptions Optional cache initialization options to configure cache behavior.
     * @returns `this` for chaining.
     */
    init(cacheInitOptions?: CacheInitOptions | null): OptionsWatcher;
    /**
     * Gets options for the specified key and feature names, validated against a schema.
     * @param cacheOptions Optional cache options to enable caching of the deserialized result. The cache must be initialized via `init` first.
     */
    getOptions<T>(key: string, featureNames: Array<string>, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions): T;
  }
}

// Extend OptionsProvider prototype with extra methods.
export const OptionsProvider = nativeBinding.OptionsProvider;
(OptionsProvider.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedResult = this[FEATURES_WITH_METADATA_CACHE_KEY];
  if (cachedResult) {
    return cachedResult;
  }

  return this[FEATURES_WITH_METADATA_CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsProvider.prototype as any).init = function (this: any, cacheInitOptions?: CacheInitOptions | null): any {
  initCache(this, cacheInitOptions);
  return this;
};
(OptionsProvider.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null, cacheOptions?: CacheOptions): any {
  return getOptionsWithCaching(this, key, featureNames, schema, preferences, cacheOptions);
};

// Extend OptionsWatcher prototype with extra methods.
export const OptionsWatcher = nativeBinding.OptionsWatcher;
(OptionsWatcher.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedTime = this[FEATURES_WITH_METADATA_CACHE_TIME_KEY];
  const lastModifiedTime = this.lastModified();
  if (cachedTime && lastModifiedTime <= cachedTime) {
    return this[FEATURES_WITH_METADATA_CACHE_KEY];
  }

  resetCaches(this);
  this[FEATURES_WITH_METADATA_CACHE_TIME_KEY] = lastModifiedTime;
  return this[FEATURES_WITH_METADATA_CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsWatcher.prototype as any).init = function (this: any, cacheInitOptions?: CacheInitOptions | null): any {
  initCache(this, cacheInitOptions);
  this[CACHE_CREATION_TIME_KEY] = this.lastModified();
  return this;
};
(OptionsWatcher.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null, cacheOptions?: CacheOptions | null): any {
  // Check cache validity for watcher - reset if files have been modified
  if (cacheOptions) {
    const lastModifiedTime = this.lastModified();
    const cacheCreationTime = this[CACHE_CREATION_TIME_KEY];

    if (!cacheCreationTime || lastModifiedTime > cacheCreationTime) {
      resetCaches(this);
      this[CACHE_CREATION_TIME_KEY] = lastModifiedTime;
    }
  }

  return getOptionsWithCaching(this, key, featureNames, schema, preferences, cacheOptions);
};
