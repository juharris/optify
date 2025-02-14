use magnus::{function, method, prelude::*, wrap, Error, Object, Ruby};
use optify::builder::OptionsProviderBuilder;
use optify::provider::OptionsProvider;
use optify::provider::GetOptionsPreferences;
use std::cell::RefCell;

#[wrap (class = "Optify::GetOptionsPreferences")]
struct MutGetOptionsPreferences(RefCell<GetOptionsPreferences>);

impl MutGetOptionsPreferences {
    fn new() -> Self {
        Self(RefCell::new(GetOptionsPreferences {
            skip_canonical_feature_name_conversion: false,
        }))
    }

    fn set_skip_canonical_feature_name_conversion(&self, value: bool) {
        self.0.borrow_mut().skip_canonical_feature_name_conversion = value;
    }

    fn skip_canonical_feature_name_conversion(&self) -> bool {
        self.0.borrow().skip_canonical_feature_name_conversion
    }
}

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

    fn get_options_json_with_preferences(
        &self,
         key: String, feature_names: Vec<String>, preferences: &MutGetOptionsPreferences) -> String {
            let _preferences = Some(optify::provider::GetOptionsPreferences {
                skip_canonical_feature_name_conversion: preferences.skip_canonical_feature_name_conversion(),
            });
        self.0.borrow().get_option_with_preferences(&key, &feature_names, &_preferences).unwrap().to_string()
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
    provider_class.define_method("get_options_json_with_preferences", method!(WrappedOptionsProvider::get_options_json_with_preferences, 3))?;

    let get_options_preferences_class = module.define_class("GetOptionsPreferences", ruby.class_object())?;
    get_options_preferences_class.define_singleton_method("new", function!(MutGetOptionsPreferences::new, 0))?;
    get_options_preferences_class.define_method("skip_canonical_feature_name_conversion=", method!(MutGetOptionsPreferences::set_skip_canonical_feature_name_conversion, 1))?;
    get_options_preferences_class.define_method("skip_canonical_feature_name_conversion", method!(MutGetOptionsPreferences::skip_canonical_feature_name_conversion, 0))?;


    Ok(())
}
