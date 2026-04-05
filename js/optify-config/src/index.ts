// TypeScript wrapper for the native bindings with additional optimizations and definitions.

// Import the generated native bindings.
import * as nativeBinding from '../index';

import {
  CacheOptions,
  CacheableInstance,
  FEATURES_WITH_METADATA_CACHE_KEY,
  FEATURES_WITH_METADATA_CACHE_TIME_KEY,
  OPTIONS_CACHE_KEY,
  CACHE_CREATION_TIME_KEY,
  getOptionsWithCaching,
  resetCaches
} from './caching';
import { TypeSchema } from './types';

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

// Re-export caching types
export { CacheOptions } from './caching';
export type { TypeSchema } from './types';

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

// Extend OptionsProvider prototype with extra methods.
export const OptionsProvider = nativeBinding.OptionsProvider;
(OptionsProvider.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedResult = this[FEATURES_WITH_METADATA_CACHE_KEY];
  if (cachedResult) {
    return cachedResult;
  }

  return this[FEATURES_WITH_METADATA_CACHE_KEY] = this._featuresWithMetadata();
};
(OptionsProvider.prototype as any).getOptions = function (this: any, key: string, featureNames: string[], schema: any, preferences?: nativeBinding.GetOptionsPreferences | null, cacheOptions?: CacheOptions | null): any {
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

  // Cache is stale, reset all caches
  resetCaches(this);
  this[FEATURES_WITH_METADATA_CACHE_TIME_KEY] = lastModifiedTime;
  return this[FEATURES_WITH_METADATA_CACHE_KEY] = this._featuresWithMetadata();
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
