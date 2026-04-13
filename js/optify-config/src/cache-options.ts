/**
 * Options for enabling caching of serialized results returned by `getOptions`
 * and for results of `getAllOptions`.
 * Pass an instance of this class to methods to enable caching.
 * Subsequent calls with equivalent inputs will return the cached result.
 */
export class CacheOptions {
	// Marker field to prevent stripping the class; instances enable caching.
	readonly enabled = true;
}
