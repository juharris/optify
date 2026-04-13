// Caching utilities for `getOptions` and `getAllOptions`.

import { LRUCache } from "lru-cache";
import * as nativeBinding from "../index";
import { CacheInitOptions } from "./cache-init-options";
import { CacheOptions } from "./cache-options";
import { TypeSchema } from "./types";

// Private cache property names (using symbols for true privacy)
export const FEATURES_WITH_METADATA_CACHE_KEY = Symbol("featuresWithMetadataCache");
export const FEATURES_WITH_METADATA_CACHE_TIME_KEY = Symbol("featuresWithMetadataCacheTime");
const OPTIONS_CACHE_KEY = Symbol("optionsCache");
export const CACHE_CREATION_TIME_KEY = Symbol("cacheCreationTime");
const CACHE_INIT_OPTIONS_KEY = Symbol("cacheInitOptions");
const SCHEMA_IDS_KEY = Symbol("schemaIds");
const SCHEMA_ID_COUNTER_KEY = Symbol("schemaIdCounter");

type OptionsCache = Map<string, NonNullable<unknown>> | LRUCache<string, NonNullable<unknown>>;

/** Instance with dynamic properties for caching. */
export interface CacheableInstance {
	_getAllOptions(featureNames: string[], preferences?: nativeBinding.GetOptionsPreferences | null): unknown;
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

/** Creates a cache key for getAllOptions caching. */
function createAllOptionsCacheKey(featureNames: string[], areConfigurableStringsEnabled: boolean): string {
	return JSON.stringify([featureNames, areConfigurableStringsEnabled]);
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
	schema: object,
): string {
	return JSON.stringify([key, featureNames, areConfigurableStringsEnabled, getSchemaId(instance, schema)]);
}

function createOptionsCache(cacheInitOptions?: CacheInitOptions | null): OptionsCache {
	if (cacheInitOptions?.maxSize !== undefined) {
		return new LRUCache<string, NonNullable<unknown>>({ max: cacheInitOptions.maxSize });
	}
	return new Map<string, NonNullable<unknown>>();
}

/**
 * Resets all caches for the instance.
 * Used by OptionsWatcher when files are modified.
 */
export function resetCaches(instance: CacheableInstance): void {
	instance[FEATURES_WITH_METADATA_CACHE_KEY] = undefined;
	instance[OPTIONS_CACHE_KEY] = createOptionsCache(instance[CACHE_INIT_OPTIONS_KEY]);
}

/**
 * Eagerly initializes the cache for the instance.
 * Optional — if not called, the cache is lazily initialized on first `getOptions` call with `cacheOptions`.
 * Call this to configure cache behavior (e.g., max size) before `getOptions`.
 * @param instance The cacheable instance to initialize.
 * @param cacheInitOptions Optional cache initialization options to configure cache behavior.
 */
export function initCache(instance: CacheableInstance, cacheInitOptions?: CacheInitOptions | null): OptionsCache {
	instance[CACHE_INIT_OPTIONS_KEY] = cacheInitOptions;
	return (instance[OPTIONS_CACHE_KEY] = createOptionsCache(cacheInitOptions));
}

/**
 * Shared implementation of getAllOptions with optional caching support.
 * Caching is enabled when `cacheOptions` is provided. The cache is lazily
 * initialized on first use if `init` was not called.
 */
export function getAllOptionsWithCaching(
	instance: CacheableInstance,
	featureNames: string[],
	preferences: nativeBinding.GetOptionsPreferences | null | undefined,
	cacheOptions: CacheOptions | null | undefined,
): unknown {
	if (cacheOptions) {
		if (preferences?.hasOverrides?.()) {
			throw new Error("Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.");
		}

		const filterPreferences = preferences || new nativeBinding.GetOptionsPreferences();
		const filteredFeatures = instance.getFilteredFeatures(featureNames, filterPreferences);

		const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

		const cache = instance[OPTIONS_CACHE_KEY] ?? initCache(instance);
		const cacheKey = createAllOptionsCacheKey(filteredFeatures, areConfigurableStringsEnabled);
		const cachedResult = cache.get(cacheKey);
		if (cachedResult !== undefined) {
			return cachedResult;
		}

		// For cache miss, create preferences that skip feature name conversion since features are already filtered.
		const cacheMissPreferences = new nativeBinding.GetOptionsPreferences();
		cacheMissPreferences.setSkipFeatureNameConversion(true);
		if (areConfigurableStringsEnabled) {
			cacheMissPreferences.enableConfigurableStrings();
		}

		const result = instance._getAllOptions(filteredFeatures, cacheMissPreferences);
		cache.set(cacheKey, result as NonNullable<unknown>);
		return result;
	}

	return instance._getAllOptions(featureNames, preferences);
}

/**
 * Shared implementation of getOptions with optional caching support.
 * Used by both `OptionsProvider` and `OptionsWatcher`.
 * Caching is enabled when `cacheOptions` is provided. The cache is lazily
 * initialized on first use if `init` was not called.
 */
export function getOptionsWithCaching<T>(
	instance: CacheableInstance,
	key: string,
	featureNames: string[],
	schema: TypeSchema<T>,
	preferences?: nativeBinding.GetOptionsPreferences | null,
	cacheOptions?: CacheOptions | null,
): T {
	if (cacheOptions) {
		if (preferences?.hasOverrides?.()) {
			throw new Error("Caching when overrides are given is not supported. Do not pass cache options when using overrides in preferences.");
		}

		const filterPreferences = preferences || new nativeBinding.GetOptionsPreferences();
		const filteredFeatures = instance.getFilteredFeatures(featureNames, filterPreferences);

		const areConfigurableStringsEnabled = preferences?.areConfigurableStringsEnabled?.() ?? false;

		const cache = instance[OPTIONS_CACHE_KEY] ?? initCache(instance);

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
