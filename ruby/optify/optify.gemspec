VERSION = "0.1.0"

Gem::Specification.new do |spec|
  spec.name = "optify"
  spec.version = VERSION
  spec.summary = "Configure your Ruby project."
  spec.homepage = "https://github.com/juharris/optify"
  spec.license = "MIT"

  spec.authors = ["Justin D. Harris"]

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