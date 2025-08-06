// TypeScript wrapper for the native bindings with additional optimizations and definitions.

// Import the generated native bindings.
import * as nativeBinding from '../index';

// Re-export types that don't need modifications.
export {
  OptionsMetadata,
  GetOptionsPreferences,
  OptionsProviderBuilder,
  OptionsWatcherListenerEvent,
  OptionsWatcherBuilder
} from '../index';

// Re-export the native classes directly
export const OptionsProvider = nativeBinding.OptionsProvider;
export const OptionsWatcher = nativeBinding.OptionsWatcher;

// Augment the native class interfaces to include our new method
declare module '../index' {
  interface OptionsProvider {
    featuresWithMetadata(): Record<string, OptionsMetadata>;
  }

  interface OptionsWatcher {
    featuresWithMetadata(): Record<string, OptionsMetadata>;
  }
}

// Private cache property names (using symbols for true privacy)
const CACHE_KEY = Symbol('featuresWithMetadataCache');
const CACHE_TIME_KEY = Symbol('featuresWithMetadataCacheTime');

// Extend OptionsProvider prototype with caching method
(OptionsProvider.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedResult = this[CACHE_KEY];
  if (cachedResult) {
    return cachedResult;
  }

  return this[CACHE_KEY] = this._featuresWithMetadata();
};

// Extend OptionsWatcher prototype with caching method that invalidates based on lastModified
(OptionsWatcher.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
  const cachedTime = this[CACHE_TIME_KEY];
  const lastModifiedTime = this.lastModified();
  if (cachedTime && lastModifiedTime <= cachedTime) {
    return this[CACHE_KEY];
  }

  this[CACHE_TIME_KEY] = lastModifiedTime;
  return this[CACHE_KEY] = this._featuresWithMetadata();
};