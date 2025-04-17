use magnus::{function, method, prelude::*, wrap, Object, Ruby};
use optify::builder::OptionsProviderBuilder;
use optify::builder::OptionsProviderBuilderTrait;
use optify::provider::GetOptionsPreferences;
use optify::provider::OptionsProvider;
use optify::provider::OptionsProviderTrait;
use optify::schema::metadata::OptionsMetadata;
use std::cell::RefCell;

#[wrap(class = "Optify::GetOptionsPreferences")]
struct MutGetOptionsPreferences(RefCell<GetOptionsPreferences>);

impl MutGetOptionsPreferences {
    fn new() -> Self {
        Self(RefCell::new(GetOptionsPreferences {
            skip_feature_name_conversion: false,
        }))
    }

    fn set_skip_feature_name_conversion(&self, value: bool) {
        self.0.borrow_mut().skip_feature_name_conversion = value;
    }

    fn skip_feature_name_conversion(&self) -> bool {
        self.0.borrow().skip_feature_name_conversion
    }
}

#[wrap(class = "Optify::OptionsProvider")]
struct WrappedOptionsProvider(RefCell<OptionsProvider>);

fn convert_metadata(metadata: &OptionsMetadata) -> String {
    serde_json::to_string(metadata).unwrap()
}

macro_rules! convert_to_str_slice {
    ($vec:expr) => {
        $vec.iter().map(|s| s.as_str()).collect::<Vec<&str>>()
    };
}

impl WrappedOptionsProvider {
    // These methods cannot accept `str`s because of how magnus works.
    // Return the JSON as a string so that it can be deserialized easily into a specific immutable class in Ruby.
    fn get_all_options_json(
        &self,
        feature_names: Vec<String>,
        preferences: &MutGetOptionsPreferences,
    ) -> Result<String, magnus::Error> {
        let _preferences = convert_preferences(preferences);
        let _features = convert_to_str_slice!(feature_names);
        Ok(self
            .0
            .borrow()
            .get_all_options(&_features, &None, &_preferences)
            .expect("features and preferences should be valid")
            .to_string())
    }

    fn get_canonical_feature_name(&self, feature_name: String) -> String {
        self.0
            .borrow()
            .get_canonical_feature_name(&feature_name)
            .unwrap()
            .to_owned()
    }
    fn get_canonical_feature_names(&self, feature_name: Vec<String>) -> Vec<String> {
        self.0
            .borrow()
            .get_canonical_feature_names(
                &feature_name
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>(),
            )
            .expect("given names should be valid")
            .into_iter()
            .map(|s| s.to_owned())
            .collect()
    }

    fn get_feature_metadata_json(&self, canonical_feature_name: String) -> Option<String> {
        self.0
            .borrow()
            .get_feature_metadata(&canonical_feature_name)
            .map(convert_metadata)
    }

    fn get_features(&self) -> Vec<String> {
        self.0
            .borrow()
            .get_features()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    // Return a string because it wasn't clear how to return a type defined in Rust despite looking at docs and trying a few examples.
    fn get_features_with_metadata_json(&self) -> String {
        serde_json::to_string(self.0.borrow().get_features_with_metadata()).unwrap()
    }

    // Return a string because it wasn't clear how to return a type defined in Rust despite looking at docs and trying a few examples.
    fn get_options_json(&self, key: String, feature_names: Vec<String>) -> String {
        let _features = convert_to_str_slice!(feature_names);
        self.0
            .borrow()
            .get_options_with_preferences(&key, &_features, &None, &None)
            .expect("key and feature names should be valid")
            .to_string()
    }

    fn get_options_json_with_preferences(
        &self,
        key: String,
        feature_names: Vec<String>,
        preferences: &MutGetOptionsPreferences,
    ) -> String {
        let _preferences = convert_preferences(preferences);
        let _features = convert_to_str_slice!(feature_names);
        self.0
            .borrow()
            .get_options_with_preferences(&key, &_features, &None, &_preferences)
            .expect("key, feature names, and preferences should be valid")
            .to_string()
    }
}

fn convert_preferences(preferences: &MutGetOptionsPreferences) -> Option<GetOptionsPreferences> {
    Some(optify::provider::GetOptionsPreferences {
        skip_feature_name_conversion: preferences.skip_feature_name_conversion(),
    })
}

#[derive(Clone)]
#[wrap(class = "Optify::OptionsProviderBuilder")]
struct WrappedOptionsProviderBuilder(RefCell<OptionsProviderBuilder>);

impl WrappedOptionsProviderBuilder {
    fn new() -> Self {
        Self(RefCell::new(OptionsProviderBuilder::new()))
    }

    fn add_directory(
        &self,
        directory: String,
    ) -> Result<WrappedOptionsProviderBuilder, magnus::Error> {
        let path = std::path::Path::new(&directory);
        self.0
            .borrow_mut()
            .add_directory(path)
            .expect("directory contents should be valid");
        Ok(self.clone())
    }

    fn build(&self) -> WrappedOptionsProvider {
        WrappedOptionsProvider(RefCell::new(
            self.0
                .borrow_mut()
                .build()
                .expect("OptionsProvider should be built successfully"),
        ))
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), magnus::Error> {
    let module = ruby.define_module("Optify")?;

    let builder_class = module.define_class("OptionsProviderBuilder", ruby.class_object())?;

    builder_class
        .define_singleton_method("new", function!(WrappedOptionsProviderBuilder::new, 0))?;
    builder_class.define_method(
        "add_directory",
        method!(WrappedOptionsProviderBuilder::add_directory, 1),
    )?;
    builder_class.define_method("build", method!(WrappedOptionsProviderBuilder::build, 0))?;

    let provider_class = module.define_class("OptionsProvider", ruby.class_object())?;
    provider_class.define_method(
        "get_all_options_json",
        method!(WrappedOptionsProvider::get_all_options_json, 2),
    )?;
    provider_class.define_method(
        "get_canonical_feature_name",
        method!(WrappedOptionsProvider::get_canonical_feature_name, 1),
    )?;
    provider_class.define_method(
        "get_canonical_feature_names",
        method!(WrappedOptionsProvider::get_canonical_feature_names, 1),
    )?;
    provider_class.define_method("features", method!(WrappedOptionsProvider::get_features, 0))?;

    // Private methods for internal use.
    provider_class.define_private_method(
        "features_with_metadata_json",
        method!(WrappedOptionsProvider::get_features_with_metadata_json, 0),
    )?;
    provider_class.define_private_method(
        "get_feature_metadata_json",
        method!(WrappedOptionsProvider::get_feature_metadata_json, 1),
    )?;
    provider_class.define_method(
        "get_options_json",
        method!(WrappedOptionsProvider::get_options_json, 2),
    )?;
    provider_class.define_method(
        "get_options_json_with_preferences",
        method!(WrappedOptionsProvider::get_options_json_with_preferences, 3),
    )?;

    let get_options_preferences_class =
        module.define_class("GetOptionsPreferences", ruby.class_object())?;
    get_options_preferences_class
        .define_singleton_method("new", function!(MutGetOptionsPreferences::new, 0))?;
    get_options_preferences_class.define_method(
        "skip_feature_name_conversion=",
        method!(
            MutGetOptionsPreferences::set_skip_feature_name_conversion,
            1
        ),
    )?;
    get_options_preferences_class.define_method(
        "skip_feature_name_conversion",
        method!(MutGetOptionsPreferences::skip_feature_name_conversion, 0),
    )?;

    Ok(())
}
