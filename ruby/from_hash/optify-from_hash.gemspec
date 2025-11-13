# frozen_string_literal: true

LIB_VERSION = '0.2.0'

Gem::Specification.new do |spec|
  spec.name = 'optify-from_hash'
  spec.version = LIB_VERSION
  spec.summary = 'Utilities for converting hashes to immutable objects.'
  spec.description = 'Helps convert hashes to immutable objects.'
  spec.homepage = 'https://github.com/juharris/optify'
  spec.license = 'MIT'
  spec.authors = ['Justin D. Harris']

  spec.required_ruby_version = '>= 3.0'

  spec.metadata = {
    'bug_tracker_uri' => 'https://github.com/juharris/optify/issues',
    # Not needed because it's the same as the source_code_uri
    # 'homepage_uri' => 'https://github.com/juharris/optify',
    'source_code_uri' => 'https://github.com/juharris/optify'
  }

  spec.files = Dir['lib/**/*.rb', 'rbi/*', 'sig/*']

  sorbet_version = '>= 0.5'
  sorbet_version_upper_bound = '< 1'
  spec.add_dependency 'sorbet-runtime', sorbet_version, sorbet_version_upper_bound

  spec.add_development_dependency 'rake'
  spec.add_development_dependency 'rbs', '~> 4.0.0.dev.4'
  spec.add_development_dependency 'sorbet', sorbet_version, sorbet_version_upper_bound
  spec.add_development_dependency 'tapioca', '~> 0.17.7'

  # Tests
  spec.add_development_dependency 'test-unit', '~> 3.6.8'
end
