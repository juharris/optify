# typed: strict
# frozen_string_literal: true

require "test/unit"
require_relative "../lib/optify"

class SampleTest < Test::Unit::TestCase
  def test_empty_build
    builder = OptionsProviderBuilder.new
    provider = builder.build
    assert_not_nil(provider)
    assert_equal(33, provider.example)
  end
end
