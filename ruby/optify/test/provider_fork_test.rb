# typed: true
# frozen_string_literal: true

require 'json'
require 'optify'
require 'test/unit'

# Verifies provider construction in processes forked after loading configurations.
class ProviderForkTest < Test::Unit::TestCase
  def test_build_after_fork
    omit('fork is not supported') unless Process.respond_to?(:fork)

    suite_path = File.expand_path('../../../tests/test_suites/simple', __dir__)
    directory = File.join(suite_path, 'configs')
    expectation = JSON.parse(File.read(File.join(suite_path, 'expectations/aliases.json')))
    Optify::OptionsProvider.build(directory)

    pid = fork do
      provider = Optify::OptionsProvider.build(directory)
      options = provider.get_options_hash('myConfig', expectation['features'])
      assert_equal(expectation['options']['myConfig'], options)
      exit! 0
    rescue StandardError => e
      warn e.full_message
      exit! 1
    end

    assert_predicate(wait_for_child(pid), :success?, 'Provider build or value resolution failed after fork')
  end

  private

  def wait_for_child(pid)
    # A Ruby timeout in the child cannot interrupt a deadlock in the native extension.
    deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + 10
    status = nil #: Process::Status?
    loop do
      result = Process.waitpid2(pid, Process::WNOHANG)
      if result
        status = result.last
        return status
      end

      flunk('Provider build in forked child did not finish within 10 seconds') if Process.clock_gettime(Process::CLOCK_MONOTONIC) >= deadline
      sleep 0.01
    end
  ensure
    unless status
      Process.kill('KILL', pid)
      Process.waitpid(pid)
    end
  end
end
