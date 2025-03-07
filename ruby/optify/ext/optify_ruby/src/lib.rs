use magnus::{
    function, method, prelude::*, wrap, DataTypeFunctions, Error, Object, Ruby, TypedData,
};
use optify::builder::OptionsProviderBuilder;
use optify::provider::GetOptionsPreferences;
use optify::provider::OptionsProvider;
use optify::schema::metadata::OptionsMetadata;
use std::cell::RefCell;
use std::collections::HashMap;

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

#[derive(DataTypeFunctions, TypedData)]
#[magnus(class = "Optify::OptionsMetadata", size, free_immediately)]
struct RubyOptionsMetadata {
    #[allow(dead_code)]
    aliases: Option<Vec<String>>,
    #[allow(dead_code)]
    details: Option<String>,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    owners: Option<String>,
}

#[derive(DataTypeFunctions, TypedData)]
#[magnus(class = "Optify::FeaturesWithMetadata", size, free_immediately)]
struct RubyFeaturesWithMetadata {
    #[allow(dead_code)]
    features: HashMap<String, RubyOptionsMetadata>,
}

fn convert_metadata(metadata: &OptionsMetadata) -> RubyOptionsMetadata {
    RubyOptionsMetadata {
        aliases: metadata.aliases.clone(),
        details: metadata.details.as_ref().map(|d| d.to_string()),
        name: metadata.name.clone().unwrap(),
        owners: metadata.owners.clone(),
    }
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
        Ok(self
            .0
            .borrow()
            .get_all_options(&feature_names, &None, &_preferences)
            .unwrap()
            .to_string())
    }

    fn get_canonical_feature_name(&self, feature_name: String) -> String {
        self.0
            .borrow()
            .get_canonical_feature_name(&feature_name)
            .unwrap()
            .to_owned()
    }

    fn get_feature_metadata_with_json_details(
        &self,
        canonical_feature_name: String,
    ) -> Option<RubyOptionsMetadata> {
        self.0
            .borrow()
            .get_feature_metadata(&canonical_feature_name)
            .map(convert_metadata)
    }

    fn get_features(&self) -> Vec<String> {
        self.0.borrow().get_features()
    }

    fn get_features_with_metadata_with_json_details(&self) -> RubyFeaturesWithMetadata {
        let mut features = HashMap::new();
        for (key, value) in self.0.borrow().get_features_with_metadata() {
            features.insert(key.to_string(), convert_metadata(value));
        }
        RubyFeaturesWithMetadata { features }
    }

    fn get_options_json(&self, key: String, feature_names: Vec<String>) -> String {
        self.0
            .borrow()
            .get_options_with_preferences(&key, &feature_names, &None, &None)
            .unwrap()
            .to_string()
    }

    fn get_options_json_with_preferences(
        &self,
        key: String,
        feature_names: Vec<String>,
        preferences: &MutGetOptionsPreferences,
    ) -> String {
        let _preferences = convert_preferences(preferences);
        self.0
            .borrow()
            .get_options_with_preferences(&key, &feature_names, &None, &_preferences)
            .unwrap()
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
        self.0.borrow_mut().add_directory(path).unwrap();
        Ok(self.clone())
    }

    fn build(&self) -> WrappedOptionsProvider {
        WrappedOptionsProvider(RefCell::new(self.0.borrow_mut().build().unwrap()))
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
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
    provider_class.define_method("features", method!(WrappedOptionsProvider::get_features, 0))?;
    provider_class.define_private_method(
        "get_feature_metadata_with_json_details",
        method!(
            WrappedOptionsProvider::get_feature_metadata_with_json_details,
            1
        ),
    )?;
    provider_class.define_private_method(
        "features_with_metadata_with_json_details",
        method!(
            WrappedOptionsProvider::get_features_with_metadata_with_json_details,
            0
        ),
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
