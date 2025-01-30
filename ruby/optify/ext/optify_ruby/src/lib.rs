use std::cell::RefCell;

use magnus::{function, method, prelude::*, wrap, Error, Object, Ruby};

use optify::{builder::OptionsProviderBuilder, provider::OptionsProvider};

#[wrap(class = "OptionsProviderBuilder")]
struct MutOptionsProviderBuilder(RefCell<OptionsProviderBuilder>);

impl MutOptionsProviderBuilder {
    fn new() -> Self {
        Self(RefCell::new(OptionsProviderBuilder::new()))
    }

    fn example(&self) -> i32 {
        3
    }
/*
    fn build(&self) -> OptionsProvider {
        self.0.borrow().build()
    }
    */
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let builder_class = ruby.define_class("OptionsProviderBuilder", ruby.class_object())?;
    builder_class.define_singleton_method("new", function!(MutOptionsProviderBuilder::new, 0))?;
    builder_class.define_method("example", method!(MutOptionsProviderBuilder::example, 0))?;
    Ok(())
}
