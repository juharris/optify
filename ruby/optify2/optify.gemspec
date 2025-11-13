# frozen_string_literal: true

VERSION = '0.0.1'

Gem::Specification.new do |spec|
  # FIXME: Find a good name
  spec.name = 'optify2'
  spec.version = VERSION
  spec.summary = 'Configure your Ruby project using JSON and YAML files that can be combined at runtime.'
  spec.description = "Simplifies getting the right configuration options for a process using pre-loaded configurations
  from files to manage options for experiments or flights."
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

  spec.add_dependency 'base64', '~> 0.2'
  spec.add_dependency 'json5', '~> 0.0.1'
  spec.add_dependency 'json-schema', '~> 5.0'
  spec.add_dependency 'liquid', '~> 5.0'
  spec.add_dependency 'listen', '~> 3.0'
  spec.add_dependency 'optify-from_hash', '~> 0.2.0'

  sorbet_version = '>= 0.5'
  sorbet_version_upper_bound = '< 1'
  spec.add_dependency 'sorbet-runtime', sorbet_version, sorbet_version_upper_bound

  spec.add_development_dependency 'rake'
  spec.add_development_dependency 'rbs', '~> 4.0.0.dev.4'
  spec.add_development_dependency 'rubocop', '~> 1.76.1'
  spec.add_development_dependency 'rubocop-sorbet', '~> 0.11.0'
  spec.add_development_dependency 'sorbet', sorbet_version, sorbet_version_upper_bound
  spec.add_development_dependency 'tapioca', '~> 0.17.7'

  # Tests
  spec.add_development_dependency 'test-unit', '~> 3.6.8'
end
