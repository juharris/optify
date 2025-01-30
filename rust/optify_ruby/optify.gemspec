VERSION = "0.1.0"

Gem::Specification.new do |spec|
  spec.name = "optify"
  spec.version = VERSION
  spec.summary = "Configure your Ruby project."
  spec.homepage = "https://github.com/juharris/optify"
  spec.license = "MIT"

  spec.authors = ["Justin D. Harris"]

  spec.extensions = ["ext/optify/extconf.rb"]

  # needed until rubygems supports Rust support is out of beta
  spec.add_dependency "rb_sys" #, "~> 0.9.39"

  # only needed when developing or packaging your gem
  spec.add_development_dependency "rake-compiler", "~> 1.2.0"
end