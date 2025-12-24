# typed: true

module LruRedux
  class ThreadSafeCache
    sig { params(max_size: Integer).void }
    def initialize(max_size); end

    sig { params(key: T::Array[T.untyped], block: T.proc.returns(T.untyped)).returns(T.untyped) }
    def getset(key, &block); end
  end
end
