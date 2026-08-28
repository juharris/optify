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
      result = provider.get_policies('feature_allowed')
      assert_not_nil(result, "Expected policies for feature_allowed from #{klass}")
      assert_instance_of(Optify::Policies, result)
      requester = result.requester
      assert_not_nil(requester)
      assert_equal(Set.new(%w[service_a service_b]), requester.allow,
                   "feature_allowed requester allow mismatch for #{klass}")
      assert_nil(requester.block)
    end
  end

  def test_get_policies_blocked
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      result = provider.get_policies('feature_blocked')
      assert_not_nil(result, "Expected policies for feature_blocked from #{klass}")
      assert_instance_of(Optify::Policies, result)
      requester = result.requester
      assert_not_nil(requester)
      assert_equal(Set.new(['untrusted_service']), requester.block,
                   "feature_blocked requester block mismatch for #{klass}")
      assert_nil(requester.allow)
    end
  end

  def test_get_policies_missing
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      assert_nil(provider.get_policies('nonexistent_feature'),
                 "Expected nil for unknown feature from #{klass}")
    end
  end

  def test_requester_in_preferences
    preferences = Optify::GetOptionsPreferences.new
    assert_nil(preferences.requester)
    preferences.requester = 'service_a'
    assert_equal('service_a', preferences.requester)
  end

  def test_raise_if_policy_denied_in_preferences_and_filtering
    PROVIDERS.each do |klass|
      provider = klass.build(POLICIES_DIR)
      preferences = Optify::GetOptionsPreferences.new
      assert_equal(false, preferences.raise_if_policy_denied)
      preferences.requester = 'unknown_service'
      preferences.raise_if_policy_denied = true
      assert_equal(true, preferences.raise_if_policy_denied)
      error = assert_raise(Optify::PolicyDeniedError) do
        provider.get_filtered_features(['feature_allowed'], preferences)
      end
      assert_equal(
        'Requester "unknown_service" is not permitted to use feature "feature_allowed".' \
        ' The requester is denied by the feature\'s policies.',
        error.message,
        "Error message mismatch for #{klass}",
      )
    end
  end
end
