# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify_ruby/base_config'

class ConfigurableString < Optify::BaseConfig
  sig { returns(String) }
  attr_reader :template

  sig { returns(T.nilable(T::Hash[String, String])) }
  attr_reader :values

  #: () -> String
  def to_str
    return template if values.nil?

    result = template

    loop do
      result_before = result
      values&.each do |key, value|
        result = result.sub(key, value)
      end

      break if result_before == result
    end

    result
  end

  #: () -> String
  def to_s
    to_str
  end
end

class ConfigurableStringTest < Test::Unit::TestCase
  def test_implicit_conversion
    c = ConfigurableString.from_hash({ template: 'wtv' })
    assert_equal(String(c), 'wtv')

    c = ConfigurableString.from_hash({ template: 'wtv', values: { 'w' => 't', 'tt' => 't' } })
    assert_equal(c.to_s, 'tv')
  end
end
