// Methods for the native `ScopedProvider` that add schema validation and use the cache of the provider or watcher.
// All of the scope logic, such as the default preferences and how overrides are applied, is in the Rust implementation.

import * as nativeBinding from "../index";
import { CacheOptions } from "./cache-options";
import { TypeSchema } from "./types";

// Only set when a cache is used because the cache of the provider or watcher is keyed by the features and needs preferences.
const DEFAULT_PREFERENCES_KEY = Symbol("defaultPreferences");
const FEATURES_KEY = Symbol("features");
const OPTIONS_CACHE_KEY = Symbol("optionsCache");

export interface ScopedProviderInstance extends nativeBinding.ScopedProvider {
	[DEFAULT_PREFERENCES_KEY]?: nativeBinding.GetOptionsPreferences;
	[FEATURES_KEY]?: string[];
	[OPTIONS_CACHE_KEY]?: Map<string, unknown>;
}

function getFeatures(instance: ScopedProviderInstance): string[] {
	return (instance[FEATURES_KEY] ??= instance._features());
}

export function initializeScopedOptionsCache(instance: ScopedProviderInstance): void {
	instance[OPTIONS_CACHE_KEY] = new Map<string, unknown>();
}

function getPreferences(
	instance: ScopedProviderInstance,
	preferences: nativeBinding.GetOptionsPreferences | null | undefined,
): nativeBinding.GetOptionsPreferences {
	if (preferences) {
		// `null` means that the given preferences can be used as they are.
		return instance._preferences(preferences) ?? preferences;
	}
	return (instance[DEFAULT_PREFERENCES_KEY] ??= instance._defaultPreferences());
}

export function getScopedAllOptions(
	instance: ScopedProviderInstance,
	preferences: nativeBinding.GetOptionsPreferences | null | undefined,
	cacheOptions: CacheOptions | null | undefined,
): any {
	if (cacheOptions) {
		return instance._registry().getAllOptions(getFeatures(instance), getPreferences(instance, preferences), cacheOptions);
	}
	return instance._getAllOptions(preferences);
}

export function getScopedOptions<T>(
	instance: ScopedProviderInstance,
	key: string,
	schema: TypeSchema<T>,
	preferences: nativeBinding.GetOptionsPreferences | null | undefined,
	cacheOptions: CacheOptions | null | undefined,
): T {
	if (!preferences && !cacheOptions) {
		const cache = instance[OPTIONS_CACHE_KEY]!;
		const cachedOptions = cache.get(key);
		// Parsed option values are expected to be truthy.
		// A falsy value is safely treated as a cache miss.
		if (cachedOptions) {
			return cachedOptions as T;
		}

		const options = schema.parse(instance._getOptions(key));
		cache.set(key, options);
		return options;
	}

	if (cacheOptions) {
		return instance._registry().getOptions(key, getFeatures(instance), schema, getPreferences(instance, preferences), cacheOptions);
	}
	return schema.parse(instance._getOptions(key, preferences));
}
