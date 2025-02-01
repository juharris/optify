# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'

class OptifyTest < Test::Unit::TestCase
  def test_empty_build
    builder = Optify::OptionsProviderBuilder.new
    provider = builder.build
    assert_not_nil(provider)
  end

  def test_get_options
    # TODO Generalize running test_suites and use a relative path from this file.
    expected = {"myArray"=>["example item 1"], "myObject"=>{"deeper"=>{"list"=>[1, 2], "wtv"=>3}, "one"=>1, "string"=>"string", "two"=>2}, "root string"=>"root string same", "root string 2"=>"gets overridden"}
    expected_json = expected.to_json
    provider = Optify::OptionsProviderBuilder.new
      .add_directory("../../tests/test_suites/simple/configs")
      .build
    config_json = provider.get_options_json("myConfig", ["A"])
    assert_equal(expected_json, config_json)

    config = provider.get_options("myConfig", ["A"])
    expected_open_struct = JSON.parse(expected_json, object_class: OpenStruct)
    assert_equal(expected_open_struct, config)
  end
end