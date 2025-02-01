# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'

class BindingsTest < Test::Unit::TestCase
  def test_empty_build
    builder = OptifyBindings::OptionsProviderBuilder.new
    provider = builder.build
    assert_not_nil(provider)
  end

  def test_get_options
    # TODO Generalize running test_suites and use a relative path from this file.
    provider = OptifyBindings::OptionsProviderBuilder.new
      .add_directory("../../tests/test_suites/simple/configs")
      .build
    config = provider.get_options_json("myConfig", ["A"])
    assert(config != nil)
    assert_equal("{\"myArray\":[\"example item 1\"],\"myObject\":{\"deeper\":{\"list\":[1,2],\"wtv\":3},\"one\":1,\"string\":\"string\",\"two\":2},\"root string\":\"root string same\",\"root string 2\":\"gets overridden\"}", config)
  end
end