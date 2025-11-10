# frozen_string_literal: true

VERSION = '1.17.6'

Gem::Specification.new do |spec|
  spec.name = 'optify-config'
  spec.version = VERSION
  spec.summary = 'Configure your Ruby project using JSON and YAML files that can be combined at runtime.'
  spec.description = "Simplifies getting the right configuration options for a process using pre-loaded configurations
  from files to manage options for experiments or flights."
  spec.homepage = 'https://github.com/juharris/optify'
  spec.license = 'MIT'

  spec.authors = ['Justin D. Harris']

  # Only certain versions are supported by Magnus: https://github.com/matsadler/magnus.
  # See notes in ruby_build_test.yml for more details about cross-compilation and source gems.
  spec.required_ruby_version = '>= 3.0'

  spec.metadata = {
    'bug_tracker_uri' => 'https://github.com/juharris/optify/issues',
    # Not needed because it's the same as the source_code_uri
    # 'homepage_uri' => 'https://github.com/juharris/optify',
    'source_code_uri' => 'https://github.com/juharris/optify'
  }

  # Cross-compilation
  # Copied from https://github.com/oxidize-rb/rb-sys/blob/main/examples/rust_reverse/rust_reverse.gemspec.
  spec.files = Dir['lib/**/*.rb', 'ext/**/*.{rs,toml,lock,rb}', 'rbi/*', 'sig/*']
  spec.bindir = 'exe'
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ['lib']

  spec.extensions = ['ext/optify_ruby/extconf.rb']

  # needed until rubygems Rust support is out of beta
  spec.add_dependency 'rb_sys', '~> 0.9.117'

  sorbet_version = '>= 0.5'
  sorbet_version_upper_bound = '< 1'
  spec.add_dependency 'sorbet-runtime', sorbet_version, sorbet_version_upper_bound

  spec.add_development_dependency 'rake-compiler', '~> 1.3.0'
  spec.add_development_dependency 'rbs', '~> 4.0.0.dev.4'
  spec.add_development_dependency 'sorbet', sorbet_version, sorbet_version_upper_bound
  spec.add_development_dependency 'tapioca', '~> 0.17.7'

  # Tests
  spec.add_development_dependency 'test-unit', '~> 3.6.8'
end
