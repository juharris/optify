/**
 * Options for enabling caching of results returned by getOptions/getAllOptions.
 * Pass an instance of this class to getOptions/getAllOptions to enable caching.
 * Subsequent calls with equivalent inputs will return the cached result.
 */
export class CacheOptions {
	// Marker field to prevent stripping the class; instances enable caching.
	readonly enabled = true;
}
