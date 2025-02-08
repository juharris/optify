# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

require 'sorbet-runtime'

class FromHashTest < Test::Unit::TestCase
  extend T::Sig

  def test_from_hash
    value = 'hello'
    hash = { 'rootString' => value, :myObject => { 'two' => 2 }, 'myObjects' => [{ two: 222 }] }
    m = MyConfig.from_hash(hash)
    assert_equal(value, m.rootString)
    assert_raises(NoMethodError) do
      T.unsafe(m).rootString = 'wtv'
    end
    assert_equal(2, m.myObject.two)
    assert_equal(222, m.myObjects[0]&.two)
  end
end
