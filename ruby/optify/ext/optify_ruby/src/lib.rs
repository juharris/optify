use magnus::{function, method, prelude::*, wrap, Error, Object, Ruby};
use optify::builder::OptionsProviderBuilder;
use optify::provider::OptionsProvider;
use std::cell::RefCell;

#[wrap(class = "Optify::RustOptionsProvider")]
struct WrappedOptionsProvider(RefCell<OptionsProvider>);

impl WrappedOptionsProvider {
    // These methods cannot accept `str`s because of how magnus works.
    // TODO Return the JSON object or maybe just keep it as a string so that it can be deserialized easily into a specific immutable class in Ruby, but then change the method name.
    fn get_options(&self, key: String, feature_names: Vec<String>) -> String {
        self.0.borrow().get_options(&key, &feature_names).unwrap().to_string()
    }
}

#[derive(Clone)]
#[wrap(class = "Optify::RustOptionsProviderBuilder")]
struct MutOptionsProviderBuilder(RefCell<OptionsProviderBuilder>);

impl MutOptionsProviderBuilder {
    fn new() -> Self {
        Self(RefCell::new(OptionsProviderBuilder::new()))
    }

    fn add_directory(&self, directory: String) -> Result<MutOptionsProviderBuilder, magnus::Error> {
        let path = std::path::Path::new(&directory);
        self.0.borrow_mut().add_directory(path).unwrap();
        Ok(self.clone())
    }

    fn build(&self) -> WrappedOptionsProvider {
        let builder = self.0.borrow();
        let provider = builder.build();
        WrappedOptionsProvider(RefCell::new(provider))
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Optify")?;
    let builder_class = module.define_class("RustOptionsProviderBuilder", ruby.class_object())?;

    builder_class.define_singleton_method("new", function!(MutOptionsProviderBuilder::new, 0))?;
    builder_class.define_method("add_directory", method!(MutOptionsProviderBuilder::add_directory, 1))?;
    builder_class.define_method("build", method!(MutOptionsProviderBuilder::build, 0))?;

    let provider_class = module.define_class("RustOptionsProvider", ruby.class_object())?;
    provider_class.define_method("get_options", method!(WrappedOptionsProvider::get_options, 2))?;
    Ok(())
}
