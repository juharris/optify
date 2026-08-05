# -*- encoding: utf-8 -*-
# stub: tapioca 0.19.1 ruby lib

Gem::Specification.new do |s|
  s.name = "tapioca".freeze
  s.version = "0.19.1".freeze

  s.required_rubygems_version = Gem::Requirement.new(">= 0".freeze) if s.respond_to? :required_rubygems_version=
  s.metadata = { "allowed_push_host" => "https://rubygems.org" } if s.respond_to? :metadata=
  s.require_paths = ["lib".freeze]
  s.authors = ["Ufuk Kayserilioglu".freeze, "Alan Wu".freeze, "Alexandre Terrasa".freeze, "Peter Zhu".freeze]
  s.bindir = "exe".freeze
  s.date = "1980-01-02"
  s.email = ["ruby@shopify.com".freeze]
  s.executables = ["tapioca".freeze]
  s.files = ["exe/tapioca".freeze]
  s.homepage = "https://github.com/Shopify/tapioca".freeze
  s.licenses = ["MIT".freeze]
  s.required_ruby_version = Gem::Requirement.new(">= 3.2".freeze)
  s.rubygems_version = "4.0.6".freeze
  s.summary = "A Ruby Interface file generator for gems, core types and the Ruby standard library".freeze

  s.installed_by_version = "3.6.7".freeze

  s.specification_version = 4

  s.add_runtime_dependency(%q<benchmark>.freeze, [">= 0".freeze])
  s.add_runtime_dependency(%q<bundler>.freeze, [">= 2.2.25".freeze])
  s.add_runtime_dependency(%q<netrc>.freeze, [">= 0.11.0".freeze])
  s.add_runtime_dependency(%q<parallel>.freeze, [">= 1.21.0".freeze])
  s.add_runtime_dependency(%q<require-hooks>.freeze, [">= 0.2.2".freeze])
  s.add_runtime_dependency(%q<rubydex>.freeze, [">= 0.1.0.beta10".freeze])
  s.add_runtime_dependency(%q<sorbet-static-and-runtime>.freeze, [">= 0.6.12698".freeze])
  s.add_runtime_dependency(%q<thor>.freeze, [">= 1.2.0".freeze])
  s.add_runtime_dependency(%q<rbi>.freeze, [">= 0.3.7".freeze])
  s.add_runtime_dependency(%q<spoom>.freeze, [">= 1.7.9".freeze])
  s.add_runtime_dependency(%q<tsort>.freeze, [">= 0".freeze])
end
