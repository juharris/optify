use magnus::{function, method, prelude::*, wrap, Error, Object, Ruby};
use optify::builder::OptionsProviderBuilder;
use optify::provider::OptionsProvider;
use std::cell::RefCell;

#[wrap(class = "Optify::OptionsProvider")]
struct WrappedOptionsProvider(RefCell<OptionsProvider>);

impl WrappedOptionsProvider {
    // These methods cannot accept `str`s because of how magnus works.
    // Return the JSON as a string so that it can be deserialized easily into a specific immutable class in Ruby.
    fn get_canonical_feature_name(&self, feature_name: String) -> String {
        self.0.borrow().get_canonical_feature_name(&feature_name).unwrap().to_owned()
    }

    fn get_options_json(&self, key: String, feature_names: Vec<String>) -> String {
        self.0.borrow().get_options(&key, &feature_names).unwrap().to_string()
    }
}

#[derive(Clone)]
#[wrap(class = "Optify::OptionsProviderBuilder")]
struct WrappedOptionsProviderBuilder(RefCell<OptionsProviderBuilder>);

impl WrappedOptionsProviderBuilder {
    fn new() -> Self {
        Self(RefCell::new(OptionsProviderBuilder::new()))
    }

    fn add_directory(&self, directory: String) -> Result<WrappedOptionsProviderBuilder, magnus::Error> {
        let path = std::path::Path::new(&directory);
        self.0.borrow_mut().add_directory(path).unwrap();
        Ok(self.clone())
    }

    fn build(&self) -> WrappedOptionsProvider {
        WrappedOptionsProvider(RefCell::new(self.0.borrow().build()))
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Optify")?;
    let builder_class = module.define_class("OptionsProviderBuilder", ruby.class_object())?;

    builder_class.define_singleton_method("new", function!(WrappedOptionsProviderBuilder::new, 0))?;
    builder_class.define_method("add_directory", method!(WrappedOptionsProviderBuilder::add_directory, 1))?;
    builder_class.define_method("build", method!(WrappedOptionsProviderBuilder::build, 0))?;

    let provider_class = module.define_class("OptionsProvider", ruby.class_object())?;
    provider_class.define_method("get_canonical_feature_name", method!(WrappedOptionsProvider::get_canonical_feature_name, 1))?;
    provider_class.define_method("get_options_json", method!(WrappedOptionsProvider::get_options_json, 2))?;
    Ok(())
}
