/**
 * Options for enabling caching of deserialized objects returned by getOptions.
 * Pass an instance of this class to getOptions to enable caching.
 * Subsequent calls with the same key, feature names, schema, and preferences
 * will return the same cached object without re-parsing.
 */
export class CacheOptions {
	// Marker field to prevent stripping the class; instances enable caching.
	readonly enabled = true;
}
