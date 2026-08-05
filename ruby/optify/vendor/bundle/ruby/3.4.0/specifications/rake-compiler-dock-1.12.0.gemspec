# -*- encoding: utf-8 -*-
# stub: rake-compiler-dock 1.12.0 ruby lib

Gem::Specification.new do |s|
  s.name = "rake-compiler-dock".freeze
  s.version = "1.12.0".freeze

  s.required_rubygems_version = Gem::Requirement.new(">= 0".freeze) if s.respond_to? :required_rubygems_version=
  s.require_paths = ["lib".freeze]
  s.authors = ["Lars Kanis".freeze]
  s.date = "1980-01-02"
  s.description = "Easy to use and reliable cross compiler environment for building Windows and Linux binary gems.\nUse rake-compiler-dock to enter an interactive shell session or add a task to your Rakefile to automate your cross build.".freeze
  s.email = ["lars@greiz-reinsdorf.de".freeze]
  s.executables = ["rake-compiler-dock".freeze]
  s.files = ["bin/rake-compiler-dock".freeze]
  s.homepage = "https://github.com/rake-compiler/rake-compiler-dock".freeze
  s.licenses = ["MIT".freeze]
  s.rubygems_version = "4.0.6".freeze
  s.summary = "Easy to use and reliable cross compiler environment for building Windows and Linux binary gems.".freeze

  s.installed_by_version = "3.6.7".freeze

  s.specification_version = 4

  s.add_development_dependency(%q<bundler>.freeze, [">= 1.7".freeze, "< 3.0".freeze])
  s.add_development_dependency(%q<rake>.freeze, [">= 12".freeze])
  s.add_development_dependency(%q<test-unit>.freeze, ["~> 3.0".freeze])
end
