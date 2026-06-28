# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify-from_hash'
require_relative 'my_config'

# Ensure that equality and other utility methods other than `from_hash` work.
module FromHashableInheritanceTest
  class TestParentObject < Optify::FromHashable
    sig { returns(T.nilable(Integer)) }
    attr_reader :num

    sig { returns(T.nilable(Hash)) }
    attr_reader :hash
  end

  class TestChildObject < TestParentObject
    sig { returns(String) }
    attr_reader :from_child
  end

  class FromHashableTest < Test::Unit::TestCase
    def test_same
      h = { num: 3, hash: { k: 1 }, from_child: 'child' }.freeze
      instance = TestChildObject.from_hash(h)
      assert_equal(h, instance.to_h)
    end
  end
end
