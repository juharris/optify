use std::cell::RefCell;

use magnus::{function, method, prelude::*, wrap, Error, Object, Ruby};

use optify::builder::OptionsProviderBuilder;

#[wrap(class = "OptionsProvider")]
struct WrappedOptionsProvider;

impl WrappedOptionsProvider {
    fn example(&self) -> i32 {
        33
    }
}

#[wrap(class = "OptionsProviderBuilder")]
struct MutOptionsProviderBuilder(RefCell<OptionsProviderBuilder>);

impl MutOptionsProviderBuilder {
    fn new() -> Self {
        Self(RefCell::new(OptionsProviderBuilder::new()))
    }

    fn build(&self) -> WrappedOptionsProvider {
        let builder = self.0.borrow();
        let provider = builder.build();
        WrappedOptionsProvider
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let builder_class = ruby.define_class("OptionsProviderBuilder", ruby.class_object())?;
    builder_class.define_singleton_method("new", function!(MutOptionsProviderBuilder::new, 0))?;
    builder_class.define_method("build", method!(MutOptionsProviderBuilder::build, 0))?;

    let provider_class = ruby.define_class("OptionsProvider", ruby.class_object())?;
    provider_class.define_method("example", method!(WrappedOptionsProvider::example, 0))?;
    Ok(())
}
