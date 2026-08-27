# typed: true
# frozen_string_literal: true

require 'json'
require 'test/unit'
require 'optify'

class PoliciesTest < Test::Unit::TestCase
  PROVIDERS = [Optify::OptionsProvider, Optify::OptionsWatcher].freeze
  POLICIES_DIR = '../../tests/test_suites/policies/configs'

  def test_get_policies_allowed
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      result = provider.get_policies_json('feature_allowed')
      assert_not_nil(result, "Expected policies JSON for feature_allowed from #{klass}")
      policies = JSON.parse(result)
      requester = policies['requester']
      requester['allowed']&.sort!
      assert_equal({ 'allowed' => %w[service_a service_b] }, requester,
                   "feature_allowed requester policies mismatch for #{klass}")
    end
  end

  def test_get_policies_blocked
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      result = provider.get_policies_json('feature_blocked')
      assert_not_nil(result, "Expected policies JSON for feature_blocked from #{klass}")
      policies = JSON.parse(result)
      assert_equal({ 'blocked' => ['untrusted_service'] }, policies['requester'],
                   "feature_blocked requester policies mismatch for #{klass}")
    end
  end

  def test_get_policies_missing
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      assert_nil(provider.get_policies_json('nonexistent_feature'),
                 "Expected nil for unknown feature from #{klass}")
    end
  end

  def test_requester_in_preferences
    preferences = Optify::GetOptionsPreferences.new
    assert_nil(preferences.requester)
    preferences.requester = 'service_a'
    assert_equal('service_a', preferences.requester)
  end
end
