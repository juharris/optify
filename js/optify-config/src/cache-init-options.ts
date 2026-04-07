/**
 * Configuration options for cache initialization.
 * Used when building providers/watchers to configure cache behavior.
 */
export class CacheInitOptions {
  /**
   * The maximum number of entries to keep in the cache.
   * When the cache is full, the least recently used entry will be evicted.
   * If not set, the cache size is unlimited.
   */
  readonly maxSize?: number;

  constructor(maxSize?: number) {
    this.maxSize = maxSize;
  }
}
