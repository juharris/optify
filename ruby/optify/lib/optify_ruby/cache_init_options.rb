# typed: strict
# frozen_string_literal: true

module Optify
  # The mode for the cache.
  module CacheMode
    # Non-thread-safe LRU cache.
    # Should be faster than `THREAD_SAFE` for single-threaded applications.
    NOT_THREAD_SAFE = :not_thread_safe #: Symbol
    # Thread-safe LRU cache.
    THREAD_SAFE = :thread_safe #: Symbol
  end

  # Options for initializing the cache.
  class CacheInitOptions
    #: Integer?
    attr_reader :max_size

    # A value from `CacheMode`.
    #
    #: Symbol
    attr_reader :mode

    # Initializes the cache options.
    # Defaults to a non-thread-safe unlimited size cache for backwards compatibility
    # with how this library was originally configured with an unbounded hash as the case.
    # @param mode A value from `CacheMode`.
    #
    #: (
    #|   ?max_size: Integer?,
    #|   ?mode: Symbol,
    #| ) -> void
    def initialize(
      max_size: nil,
      mode: CacheMode::NOT_THREAD_SAFE
    )
      @max_size = max_size
      @mode = mode
    end
  end
end
