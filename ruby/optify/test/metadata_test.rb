# frozen_string_literal: true
# typed: true

require 'json'
require 'test/unit'
require_relative '../lib/optify'
require_relative 'my_config'

require 'sorbet-runtime'

class OptifyTest < Test::Unit::TestCase
  extend T::Sig

  def test_features_with_metadata
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory('../../tests/test_suites/simple/configs')
                                             .build
    a_metadata = provider.get_feature_metadata('feature_A')
    assert_not_nil(a_metadata)
    assert_equal('feature_A', a_metadata&.name)
    assert_equal(['a'], a_metadata&.aliases)
    assert_equal('The file is for testing.', a_metadata&.details)
    assert_equal('a-team@company.com', a_metadata&.owners)

    b_metadata = provider.get_feature_metadata('feature_B/initial')
    assert_not_nil(b_metadata)
    assert_equal('feature_B/initial', b_metadata&.name)
    assert_equal(['b'], b_metadata&.aliases)
    assert_equal({ 'description' => 'This is a description of the feature.' }, b_metadata&.details)
    assert_equal('team-b@company.com', b_metadata&.owners)

    all_metadata = provider.features_with_metadata
    assert_equal(3, all_metadata.size)
    assert_equal(a_metadata&.aliases, all_metadata['feature_A']&.aliases)
    assert_equal(a_metadata&.name, all_metadata['feature_A']&.name)
    assert_equal(b_metadata&.aliases, all_metadata['feature_B/initial']&.aliases)
    assert_equal(b_metadata&.name, all_metadata['feature_B/initial']&.name)
  end
end
