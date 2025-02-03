VERSION = "0.2.0"

Gem::Specification.new do |spec|
  spec.name = "optify-config"
  spec.version = VERSION
  spec.summary = "Configure your Ruby project using JSON and YAML files that can be combined at runtime."
  spec.description = "Simplifies getting the right configuration options for a process using pre-loaded configurations from files to manage options for experiments or flights."
  spec.homepage = "https://github.com/juharris/optify"
  spec.license = "MIT"

  spec.authors = ["Justin D. Harris"]

  spec.metadata = {
    'source_code_uri' => 'https://github.com/juharris/optify',
    'bug_tracker_uri' => 'https://github.com/juharris/optify/issues',
  }

  # Cross-compilation
  # Copied from https://github.com/oxidize-rb/rb-sys/blob/main/examples/rust_reverse/rust_reverse.gemspec.
  spec.files = Dir["lib/**/*.rb", "ext/**/*.{rs,toml,lock,rb}"]
  spec.bindir = "exe"
  spec.executables = spec.files.grep(%r{\Aexe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  spec.extensions = ["ext/optify_ruby/extconf.rb"]

  # needed until rubygems supports Rust support is out of beta
  spec.add_dependency "rb_sys" , "~> 0.9.109"

  spec.add_dependency "sorbet-runtime"

  spec.add_development_dependency "rake-compiler", "~> 1.2.9"
  spec.add_development_dependency "sorbet"
  spec.add_development_dependency "tapioca"

  # Tests
  spec.add_development_dependency "test-unit"
end