// TypeScript wrapper for the native bindings with additional optimizations and definitions.

// Import the generated native bindings.
import * as nativeBinding from "../index";

import { CacheInitOptions } from "./cache-init-options";
import { CacheOptions } from "./cache-options";
import {
	CACHE_CREATION_TIME_KEY,
	FEATURES_WITH_METADATA_CACHE_KEY,
	FEATURES_WITH_METADATA_CACHE_TIME_KEY,
	getAllOptionsWithCaching,
	getOptionsWithCaching,
	initCache,
	resetCaches,
	resetWatcherCachesIfModified,
} from "./caching";
import { getScopedAllOptions, getScopedOptions, initializeScopedOptionsCache } from "./scoped-provider";
import { TypeSchema } from "./types";

// Re-export types that don't need modifications.
export { BuilderOptions, GetOptionsPreferences, OptionsWatcherListenerEvent, WatcherOptions } from "../index";

export type Feature = Omit<nativeBinding.Feature, "metadata"> & {
	metadata: OptionsMetadata;
};
export type OptionsMetadata = nativeBinding.OptionsMetadata;

// Re-export the native classes directly
export type OptionsProvider = nativeBinding.OptionsProvider;
export type OptionsWatcher = nativeBinding.OptionsWatcher;

export { CacheInitOptions };
export { CacheOptions };
export type {
	AllowRequesterPolicy,
	BlockRequesterPolicy,
	OptionAnalysis,
	OptionOrigin,
	OptionSource,
	Policies,
	RequesterPolicy,
} from "../index";
export type { TypeSchema } from "./types";

// An instanceof check can be spoofed by changing another native object's prototype.
const metadataInstances = new WeakSet<nativeBinding.OptionsMetadata>();
const nativeMetadataToJson = nativeBinding.OptionsMetadata.prototype.toJson;

function registerMetadata(metadata: nativeBinding.OptionsMetadata): void {
	metadataInstances.add(metadata);
}

export const OptionsMetadata = new Proxy(nativeBinding.OptionsMetadata, {
	construct(target, args, newTarget) {
		const metadata: nativeBinding.OptionsMetadata = Reflect.construct(target, args, newTarget);
		registerMetadata(metadata);
		return metadata;
	},
});

export const Feature = {
	/** Deserializes a feature and its metadata from JSON. */
	fromJson(json: string): Feature {
		const feature = nativeBinding._featureFromJson(json);
		registerMetadata(feature.metadata);
		return feature;
	},
};

function registerMetadataResults(metadata: Record<string, nativeBinding.OptionsMetadata>): Record<string, nativeBinding.OptionsMetadata> {
	for (const value of Object.values(metadata)) {
		registerMetadata(value);
	}
	return metadata;
}

const providerFeaturesWithMetadata = nativeBinding.OptionsProvider.prototype._featuresWithMetadata;
nativeBinding.OptionsProvider.prototype._featuresWithMetadata = function () {
	return registerMetadataResults(providerFeaturesWithMetadata.call(this));
};
const watcherFeaturesWithMetadata = nativeBinding.OptionsWatcher.prototype._featuresWithMetadata;
nativeBinding.OptionsWatcher.prototype._featuresWithMetadata = function () {
	return registerMetadataResults(watcherFeaturesWithMetadata.call(this));
};

function toNativeFeature(feature: Feature): nativeBinding.Feature {
	const metadata = feature.metadata;
	if (!metadataInstances.has(metadata)) {
		throw new TypeError("Feature metadata must be an OptionsMetadata instance");
	}
	return { ...feature, metadata: nativeMetadataToJson.call(metadata) };
}

export class OptionsProviderBuilder extends nativeBinding.OptionsProviderBuilder {
	addFeature(feature: Feature): void {
		super.addFeature(toNativeFeature(feature));
	}

	addFeatures(features: Array<Feature>): void {
		super.addFeatures(features.map(toNativeFeature));
	}
}

export class OptionsWatcherBuilder extends nativeBinding.OptionsWatcherBuilder {
	addFeature(feature: Feature): void {
		super.addFeature(toNativeFeature(feature));
	}

	addFeatures(features: Array<Feature>): void {
		super.addFeatures(features.map(toNativeFeature));
	}
}

// Augment the native class interfaces to include our new method
declare module "../index" {
	interface OptionsProvider {
		/** Returns a map of all the canonical feature names to their metadata. */
		featuresWithMetadata(): Record<string, OptionsMetadata>;
		/**
		 * Eagerly initializes the cache. Optional — the cache is lazily initialized on first
		 * `getOptions` call with `cacheOptions` if not called. Use this to configure cache behavior
		 * (e.g., max size) before `getOptions`.
		 * @param cacheInitOptions Optional cache initialization options to configure cache behavior.
		 * @returns `this` for chaining.
		 */
		init(cacheInitOptions?: CacheInitOptions | null): OptionsProvider;
		/**
		 * Gets options for the specified key and feature names, validated against a schema.
		 * @param cacheOptions Optional cache options to enable caching of the deserialized result.
		 */
		getOptions<T>(
			key: string,
			featureNames: Array<string>,
			schema: TypeSchema<T>,
			preferences?: GetOptionsPreferences | null,
			cacheOptions?: CacheOptions,
		): T;
		/**
		 * Gets all options for the specified feature names.
		 * @param cacheOptions Optional cache options to enable caching of the result.
		 */
		getAllOptions(featureNames: Array<string>, preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions): any;
	}

	interface OptionsWatcher {
		/** Returns a map of all the canonical feature names to their metadata. */
		featuresWithMetadata(): Record<string, OptionsMetadata>;
		/**
		 * Eagerly initializes the cache. Optional — the cache is lazily initialized on first
		 * `getOptions` call with `cacheOptions` if not called. Use this to configure cache behavior
		 * (e.g., max size) before `getOptions`.
		 * @param cacheInitOptions Optional cache initialization options to configure cache behavior.
		 * @returns `this` for chaining.
		 */
		init(cacheInitOptions?: CacheInitOptions | null): OptionsWatcher;
		/**
		 * Gets options for the specified key and feature names, validated against a schema.
		 * @param cacheOptions Optional cache options to enable caching of the deserialized result.
		 */
		getOptions<T>(
			key: string,
			featureNames: Array<string>,
			schema: TypeSchema<T>,
			preferences?: GetOptionsPreferences | null,
			cacheOptions?: CacheOptions,
		): T;
		/**
		 * Gets all options for the specified feature names.
		 * @param cacheOptions Optional cache options to enable caching of the result.
		 */
		getAllOptions(featureNames: Array<string>, preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions): any;
	}

	interface ScopedProvider {
		/**
		 * Gets all options using this scope's features and overrides.
		 * @param preferences Optional preferences to use instead of the defaults.
		 * They are not changed: if this scope has overrides, then a copy is used with this scope's overrides.
		 * @param cacheOptions Optional cache options to enable caching of the result by the provider or watcher.
		 * Caching is not supported when there are overrides.
		 */
		getAllOptions(preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions): any;
		/**
		 * Gets options for a key using this scope's features and overrides, validated against a schema.
		 * @param preferences Optional preferences to use instead of the defaults.
		 * They are not changed: if this scope has overrides, then a copy is used with this scope's overrides.
		 * @param cacheOptions Optional cache options to enable caching of the deserialized result by the provider or watcher.
		 * Caching is not supported when there are overrides.
		 */
		getOptions<T>(key: string, schema: TypeSchema<T>, preferences?: GetOptionsPreferences | null, cacheOptions?: CacheOptions): T;
	}
}

// Wrap the native scoped provider so its parsed-result cache is allocated with each instance.
export class ScopedProvider extends nativeBinding.ScopedProvider {
	constructor(registry: nativeBinding.OptionsProvider | nativeBinding.OptionsWatcher, features: string[], overrides?: unknown | null) {
		super(registry, features, overrides);
		initializeScopedOptionsCache(this);
	}

	static deserialize(registry: nativeBinding.OptionsProvider | nativeBinding.OptionsWatcher, value: unknown): ScopedProvider {
		const scoped = nativeBinding.ScopedProvider.deserialize(registry, value);
		Object.setPrototypeOf(scoped, ScopedProvider.prototype);
		initializeScopedOptionsCache(scoped);
		return scoped;
	}
}

// Extend ScopedProvider prototype with extra methods.
(ScopedProvider.prototype as any).getAllOptions = function (
	this: any,
	preferences?: nativeBinding.GetOptionsPreferences | null,
	cacheOptions?: CacheOptions | null,
): any {
	return getScopedAllOptions(this, preferences, cacheOptions);
};
(ScopedProvider.prototype as any).getOptions = function (
	this: any,
	key: string,
	schema: any,
	preferences?: nativeBinding.GetOptionsPreferences | null,
	cacheOptions?: CacheOptions | null,
): any {
	return getScopedOptions(this, key, schema, preferences, cacheOptions);
};

// Extend OptionsProvider prototype with extra methods.
export const OptionsProvider = nativeBinding.OptionsProvider;
(OptionsProvider.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
	const cachedResult = this[FEATURES_WITH_METADATA_CACHE_KEY];
	if (cachedResult) {
		return cachedResult;
	}

	return (this[FEATURES_WITH_METADATA_CACHE_KEY] = this._featuresWithMetadata());
};
(OptionsProvider.prototype as any).init = function (this: any, cacheInitOptions?: CacheInitOptions | null): any {
	initCache(this, cacheInitOptions);
	return this;
};
(OptionsProvider.prototype as any).getAllOptions = function (
	this: any,
	featureNames: string[],
	preferences?: nativeBinding.GetOptionsPreferences | null,
	cacheOptions?: CacheOptions | null,
): any {
	return getAllOptionsWithCaching(this, featureNames, preferences, cacheOptions);
};
(OptionsProvider.prototype as any).getOptions = function (
	this: any,
	key: string,
	featureNames: string[],
	schema: any,
	preferences?: nativeBinding.GetOptionsPreferences | null,
	cacheOptions?: CacheOptions,
): any {
	return getOptionsWithCaching(this, key, featureNames, schema, preferences, cacheOptions);
};

// Extend OptionsWatcher prototype with extra methods.
export const OptionsWatcher = nativeBinding.OptionsWatcher;
(OptionsWatcher.prototype as any).featuresWithMetadata = function (this: any): Record<string, nativeBinding.OptionsMetadata> {
	const cachedTime = this[FEATURES_WITH_METADATA_CACHE_TIME_KEY];
	const lastModifiedTime = this.lastModified();
	if (cachedTime && lastModifiedTime <= cachedTime) {
		const cachedResult = this[FEATURES_WITH_METADATA_CACHE_KEY];
		if (cachedResult) {
			return cachedResult;
		}
	}

	// Only reset the options cache if we previously had cached metadata and files have changed.
	// On the first call (cachedTime is undefined) there is nothing stale to evict.
	if (cachedTime) {
		resetCaches(this);
	}
	this[FEATURES_WITH_METADATA_CACHE_TIME_KEY] = lastModifiedTime;
	return (this[FEATURES_WITH_METADATA_CACHE_KEY] = this._featuresWithMetadata());
};
(OptionsWatcher.prototype as any).init = function (this: any, cacheInitOptions?: CacheInitOptions | null): any {
	initCache(this, cacheInitOptions);
	this[CACHE_CREATION_TIME_KEY] = this.lastModified();
	return this;
};
(OptionsWatcher.prototype as any).getAllOptions = function (
	this: any,
	featureNames: string[],
	preferences?: nativeBinding.GetOptionsPreferences | null,
	cacheOptions?: CacheOptions | null,
): any {
	if (cacheOptions) {
		resetWatcherCachesIfModified(this);
	}

	return getAllOptionsWithCaching(this, featureNames, preferences, cacheOptions);
};
(OptionsWatcher.prototype as any).getOptions = function (
	this: any,
	key: string,
	featureNames: string[],
	schema: any,
	preferences?: nativeBinding.GetOptionsPreferences | null,
	cacheOptions?: CacheOptions | null,
): any {
	if (cacheOptions) {
		resetWatcherCachesIfModified(this);
	}

	return getOptionsWithCaching(this, key, featureNames, schema, preferences, cacheOptions);
};
