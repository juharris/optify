use magnus::{function, method, prelude::*, wrap, Object, Ruby};
use optify::builder::OptionsProviderBuilder;
use optify::builder::OptionsRegistryBuilder;
use optify::builder::OptionsWatcherBuilder;
use optify::convert_to_str_slice;
use optify::provider::GetOptionsPreferences;
use optify::provider::OptionsProvider;
use optify::provider::OptionsRegistry;
use optify::provider::OptionsWatcher;
use optify::schema::metadata::OptionsMetadata;
use std::cell::RefCell;

#[derive(Clone)]
#[wrap(class = "Optify::GetOptionsPreferences")]
struct MutGetOptionsPreferences(RefCell<GetOptionsPreferences>);

impl MutGetOptionsPreferences {
    fn new() -> Self {
        Self(RefCell::new(GetOptionsPreferences {
            overrides_json: None,
            skip_feature_name_conversion: false,
        }))
    }

    // Overrides Section
    fn has_overrides(&self) -> bool {
        self.0.borrow().overrides_json.is_some()
    }

    fn set_overrides_json(&self, overrides: Option<String>) {
        self.0.borrow_mut().overrides_json = overrides;
    }

    fn get_overrides_json(&self) -> Option<String> {
        self.0.borrow().overrides_json.clone()
    }

    // Skip Feature Name Conversion Section
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

impl WrappedOptionsProvider {
    // These methods cannot accept `str`s because of how magnus works.
    // Return the JSON as a string so that it can be deserialized easily into a specific immutable class in Ruby.
    fn get_all_options_json(
        ruby: &Ruby,
        rb_self: &Self,
        feature_names: Vec<String>,
        preferences: &MutGetOptionsPreferences,
    ) -> Result<String, magnus::Error> {
        let _preferences = convert_preferences(preferences);
        let features = convert_to_str_slice!(feature_names);
        match rb_self
            .0
            .borrow()
            .get_all_options(&features, &None, &_preferences)
        {
            Ok(options) => Ok(options.to_string()),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }

    fn get_canonical_feature_name(
        ruby: &Ruby,
        rb_self: &Self,
        feature_name: String,
    ) -> Result<String, magnus::Error> {
        rb_self
            .0
            .borrow()
            .get_canonical_feature_name(&feature_name)
            .map_err(|e| magnus::Error::new(ruby.exception_exception(), e))
    }

    fn get_canonical_feature_names(
        ruby: &Ruby,
        rb_self: &Self,
        feature_names: Vec<String>,
    ) -> Result<Vec<String>, magnus::Error> {
        let features = convert_to_str_slice!(feature_names);
        rb_self
            .0
            .borrow()
            .get_canonical_feature_names(&features)
            .map_err(|e| magnus::Error::new(ruby.exception_exception(), e))
    }

    fn get_feature_metadata_json(&self, canonical_feature_name: String) -> Option<String> {
        self.0
            .borrow()
            .get_feature_metadata(&canonical_feature_name)
            .map(|metadata| convert_metadata(&metadata))
    }

    fn get_features(&self) -> Vec<String> {
        self.0.borrow().get_features()
    }

    // Return a string because it wasn't clear how to return a type defined in Rust despite looking at docs and trying a few examples.
    fn get_features_with_metadata_json(&self) -> String {
        serde_json::to_string(&self.0.borrow().get_features_with_metadata()).unwrap()
    }

    // Return a string because it wasn't clear how to return a type defined in Rust despite looking at docs and trying a few examples.
    fn get_options_json(
        ruby: &Ruby,
        rb_self: &Self,
        key: String,
        feature_names: Vec<String>,
    ) -> Result<String, magnus::Error> {
        let features = convert_to_str_slice!(feature_names);
        match rb_self
            .0
            .borrow()
            .get_options_with_preferences(&key, &features, &None, &None)
        {
            Ok(options) => Ok(options.to_string()),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }

    fn get_options_json_with_preferences(
        ruby: &Ruby,
        rb_self: &Self,
        key: String,
        feature_names: Vec<String>,
        preferences: &MutGetOptionsPreferences,
    ) -> Result<String, magnus::Error> {
        let _preferences = convert_preferences(preferences);
        let features = convert_to_str_slice!(feature_names);
        match rb_self
            .0
            .borrow()
            .get_options_with_preferences(&key, &features, &None, &_preferences)
        {
            Ok(options) => Ok(options.to_string()),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }
}

fn convert_preferences(preferences: &MutGetOptionsPreferences) -> Option<GetOptionsPreferences> {
    Some(optify::provider::GetOptionsPreferences {
        overrides_json: preferences.get_overrides_json(),
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
        ruby: &Ruby,
        rb_self: &Self,
        directory: String,
    ) -> Result<WrappedOptionsProviderBuilder, magnus::Error> {
        let path = std::path::Path::new(&directory);
        match rb_self.0.borrow_mut().add_directory(path) {
            Ok(builder) => Ok(WrappedOptionsProviderBuilder(RefCell::new(builder.clone()))),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }

    fn build(ruby: &Ruby, rb_self: &Self) -> Result<WrappedOptionsProvider, magnus::Error> {
        match rb_self.0.borrow_mut().build() {
            Ok(provider) => Ok(WrappedOptionsProvider(RefCell::new(provider))),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }
}

#[wrap(class = "Optify::OptionsWatcher")]
struct WrappedOptionsWatcher(RefCell<OptionsWatcher>);

impl WrappedOptionsWatcher {
    fn get_all_options_json(
        ruby: &Ruby,
        rb_self: &Self,
        feature_names: Vec<String>,
        preferences: &MutGetOptionsPreferences,
    ) -> Result<String, magnus::Error> {
        let _preferences = convert_preferences(preferences);
        let features = convert_to_str_slice!(feature_names);
        match rb_self
            .0
            .borrow()
            .get_all_options(&features, &None, &_preferences)
        {
            Ok(options) => Ok(options.to_string()),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }

    fn get_canonical_feature_name(
        ruby: &Ruby,
        rb_self: &Self,
        feature_name: String,
    ) -> Result<String, magnus::Error> {
        rb_self
            .0
            .borrow()
            .get_canonical_feature_name(&feature_name)
            .map_err(|e| magnus::Error::new(ruby.exception_exception(), e))
    }

    fn get_canonical_feature_names(
        ruby: &Ruby,
        rb_self: &Self,
        feature_names: Vec<String>,
    ) -> Result<Vec<String>, magnus::Error> {
        let features = convert_to_str_slice!(feature_names);
        rb_self
            .0
            .borrow()
            .get_canonical_feature_names(&features)
            .map_err(|e| magnus::Error::new(ruby.exception_exception(), e))
    }

    fn get_feature_metadata_json(&self, canonical_feature_name: String) -> Option<String> {
        self.0
            .borrow()
            .get_feature_metadata(&canonical_feature_name)
            .map(|metadata| convert_metadata(&metadata))
    }

    fn get_features(&self) -> Vec<String> {
        self.0.borrow().get_features()
    }

    fn get_features_with_metadata_json(&self) -> String {
        serde_json::to_string(&self.0.borrow().get_features_with_metadata()).unwrap()
    }

    fn get_options_json(
        ruby: &Ruby,
        rb_self: &Self,
        key: String,
        feature_names: Vec<String>,
    ) -> Result<String, magnus::Error> {
        let features = convert_to_str_slice!(feature_names);
        match rb_self
            .0
            .borrow()
            .get_options_with_preferences(&key, &features, &None, &None)
        {
            Ok(options) => Ok(options.to_string()),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }

    fn get_options_json_with_preferences(
        ruby: &Ruby,
        rb_self: &Self,
        key: String,
        feature_names: Vec<String>,
        preferences: &MutGetOptionsPreferences,
    ) -> Result<String, magnus::Error> {
        let _preferences = convert_preferences(preferences);
        let features = convert_to_str_slice!(feature_names);
        match rb_self
            .0
            .borrow()
            .get_options_with_preferences(&key, &features, &None, &_preferences)
        {
            Ok(options) => Ok(options.to_string()),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }

    fn last_modified(&self) -> std::time::SystemTime {
        self.0.borrow().last_modified()
    }
}

#[derive(Clone)]
#[wrap(class = "Optify::OptionsWatcherBuilder")]
struct WrappedOptionsWatcherBuilder(RefCell<OptionsWatcherBuilder>);

impl WrappedOptionsWatcherBuilder {
    fn new() -> Self {
        Self(RefCell::new(OptionsWatcherBuilder::new()))
    }

    fn add_directory(
        ruby: &Ruby,
        rb_self: &Self,
        directory: String,
    ) -> Result<WrappedOptionsWatcherBuilder, magnus::Error> {
        let path = std::path::Path::new(&directory);
        match rb_self.0.borrow_mut().add_directory(path) {
            Ok(builder) => Ok(WrappedOptionsWatcherBuilder(RefCell::new(builder.clone()))),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
    }

    fn build(ruby: &Ruby, rb_self: &Self) -> Result<WrappedOptionsWatcher, magnus::Error> {
        match rb_self.0.borrow_mut().build() {
            Ok(provider) => Ok(WrappedOptionsWatcher(RefCell::new(provider))),
            Err(e) => Err(magnus::Error::new(ruby.exception_exception(), e)),
        }
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
    provider_class.define_method("features", method!(WrappedOptionsProvider::get_features, 0))?;
    provider_class.define_method(
        "get_all_options_json",
        method!(WrappedOptionsProvider::get_all_options_json, 2),
    )?;
    provider_class.define_method(
        "get_canonical_feature_name",
        method!(WrappedOptionsProvider::get_canonical_feature_name, 1),
    )?;
    provider_class.define_method(
        "get_options_json",
        method!(WrappedOptionsProvider::get_options_json, 2),
    )?;
    provider_class.define_method(
        "get_options_json_with_preferences",
        method!(WrappedOptionsProvider::get_options_json_with_preferences, 3),
    )?;

    // Private methods for internal use.
    provider_class.define_private_method(
        "_get_canonical_feature_names",
        method!(WrappedOptionsProvider::get_canonical_feature_names, 1),
    )?;
    provider_class.define_private_method(
        "features_with_metadata_json",
        method!(WrappedOptionsProvider::get_features_with_metadata_json, 0),
    )?;
    provider_class.define_private_method(
        "get_feature_metadata_json",
        method!(WrappedOptionsProvider::get_feature_metadata_json, 1),
    )?;

    let get_options_preferences_class =
        module.define_class("GetOptionsPreferences", ruby.class_object())?;
    get_options_preferences_class
        .define_singleton_method("new", function!(MutGetOptionsPreferences::new, 0))?;
    get_options_preferences_class
        .define_method("dup", method!(MutGetOptionsPreferences::clone, 0))?;
    get_options_preferences_class.define_method(
        "overrides?",
        method!(MutGetOptionsPreferences::has_overrides, 0),
    )?;
    get_options_preferences_class.define_method(
        "overrides_json=",
        method!(MutGetOptionsPreferences::set_overrides_json, 1),
    )?;
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

    let watcher_builder_class =
        module.define_class("OptionsWatcherBuilder", ruby.class_object())?;
    watcher_builder_class
        .define_singleton_method("new", function!(WrappedOptionsWatcherBuilder::new, 0))?;
    watcher_builder_class.define_method(
        "add_directory",
        method!(WrappedOptionsWatcherBuilder::add_directory, 1),
    )?;
    watcher_builder_class
        .define_method("build", method!(WrappedOptionsWatcherBuilder::build, 0))?;

    let watcher_class = module.define_class("OptionsWatcher", ruby.class_object())?;
    watcher_class.define_method("features", method!(WrappedOptionsWatcher::get_features, 0))?;
    watcher_class.define_method(
        "get_all_options_json",
        method!(WrappedOptionsWatcher::get_all_options_json, 2),
    )?;
    watcher_class.define_method(
        "get_canonical_feature_name",
        method!(WrappedOptionsWatcher::get_canonical_feature_name, 1),
    )?;
    watcher_class.define_method(
        "get_options_json",
        method!(WrappedOptionsWatcher::get_options_json, 2),
    )?;
    watcher_class.define_method(
        "get_options_json_with_preferences",
        method!(WrappedOptionsWatcher::get_options_json_with_preferences, 3),
    )?;
    watcher_class.define_method(
        "last_modified",
        method!(WrappedOptionsWatcher::last_modified, 0),
    )?;

    // Private methods for internal use.
    watcher_class.define_private_method(
        "features_with_metadata_json",
        method!(WrappedOptionsWatcher::get_features_with_metadata_json, 0),
    )?;
    watcher_class.define_private_method(
        "_get_canonical_feature_names",
        method!(WrappedOptionsWatcher::get_canonical_feature_names, 1),
    )?;
    watcher_class.define_private_method(
        "get_feature_metadata_json",
        method!(WrappedOptionsWatcher::get_feature_metadata_json, 1),
    )?;

    Ok(())
}
