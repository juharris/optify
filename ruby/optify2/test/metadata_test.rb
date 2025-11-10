# frozen_string_literal: true
# typed: true

require 'test/unit'
require_relative '../lib/optify'

class OptifyTest < Test::Unit::TestCase
  def test_features_with_metadata
    provider = Optify::OptionsProviderBuilder.new
                                             .add_directory('../../tests/test_suites/simple/configs')
                                             .build
    a_metadata = provider.get_feature_metadata('feature_A')
    expected_metadata = Optify::OptionsMetadata.from_hash(
      {
        name: 'feature_A',
        aliases: ['a'],
        details: 'The file is for testing.',
        owners: 'a-team@company.com'
      }
    )
    assert_equal(expected_metadata, a_metadata)

    b_metadata = provider.get_feature_metadata('feature_B/initial')
    expected_metadata = Optify::OptionsMetadata.from_hash(
      {
        name: 'feature_B/initial',
        aliases: ['b'],
        details: { 'description' => 'This is a description of the feature.' },
        owners: 'team-b@company.com'
      }
    )
    assert_equal(expected_metadata, b_metadata)

    all_metadata = provider.features_with_metadata
    assert_equal(3, all_metadata.size)
    assert_equal(a_metadata&.aliases, all_metadata['feature_A']&.aliases)
    assert_equal(a_metadata&.name, all_metadata['feature_A']&.name)
    assert_equal(b_metadata&.aliases, all_metadata['feature_B/initial']&.aliases)
    assert_equal(b_metadata&.name, all_metadata['feature_B/initial']&.name)
    assert_same(all_metadata, provider.features_with_metadata)
  end
end
