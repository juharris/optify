use magnus::{function, method, prelude::*, wrap, Error, Object, Ruby};
use optify::builder::OptionsProviderBuilder;
use optify::provider::OptionsProvider;
use std::cell::RefCell;

#[wrap(class = "OptionsProvider")]
struct WrappedOptionsProvider(RefCell<OptionsProvider>);

impl WrappedOptionsProvider {
    // These methods cannot accept `str`s because of how magnus works.
    // TODO Return the JSON object.
    fn get_options(&self, key: String, feature_names: Vec<String>) -> String {
        self.0.borrow().get_options(&key, &feature_names).unwrap().to_string()
    }
}

#[wrap(class = "OptionsProviderBuilder")]
struct MutOptionsProviderBuilder(RefCell<OptionsProviderBuilder>);

impl MutOptionsProviderBuilder {
    fn new() -> Self {
        Self(RefCell::new(OptionsProviderBuilder::new()))
    }

    // TODO Make fluent and return MutOptionsProviderBuilder.
    fn add_directory(&self, directory: String) -> Result<(), magnus::Error> {
        let path = std::path::Path::new(&directory);
        // TODO Try to avoid cloning and to support copying/moving if necessary.
        let builder = self.0.borrow_mut().clone();
        self.0.replace(builder.add_directory(path).unwrap());
        Ok(())
    }

    fn build(&self) -> WrappedOptionsProvider {
        let builder = self.0.borrow();
        let provider = builder.build();
        WrappedOptionsProvider(RefCell::new(provider))
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let builder_class = ruby.define_class("OptionsProviderBuilder", ruby.class_object())?;
    builder_class.define_singleton_method("new", function!(MutOptionsProviderBuilder::new, 0))?;
    builder_class.define_method("add_directory", method!(MutOptionsProviderBuilder::add_directory, 1))?;
    builder_class.define_method("build", method!(MutOptionsProviderBuilder::build, 0))?;

    let provider_class = ruby.define_class("OptionsProvider", ruby.class_object())?;
    provider_class.define_method("get_options", method!(WrappedOptionsProvider::get_options, 2))?;
    Ok(())
}
