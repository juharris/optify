use liquid::ObjectView;
use liquid::ValueView;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ReplacementObject {
    File { file: String },
    Liquid { liquid: String },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ReplacementValue {
    String(String),
    Object(ReplacementObject),
}

/// Helps build a string by components declared in files.
/// Parsed from a `serde_json::Value`.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ConfigurableString {
    pub base: ReplacementValue,
    pub arguments: Option<HashMap<String, ReplacementValue>>,
}

/// Mapping from relative file paths to their contents.
pub type LoadedFiles = HashMap<String, String>;

/// Dynamic object that resolves values on demand.
struct DynamicArguments<'a> {
    // Cache resolved values so that we can return a reference to the value.
    // Use RefCell to allow interior mutability because of the signature for `get` in the trait.
    // An alternative is to pre-compute all of the possible values,
    // but some configured values might not be used or be able to be rendered.
    cache: RefCell<HashMap<String, liquid::model::Value>>,
    errors: RefCell<Vec<String>>,
    files: &'a LoadedFiles,
    parser: liquid::Parser,
    arguments: &'a HashMap<String, ReplacementValue>,
}

impl<'a> DynamicArguments<'a> {
    fn new(arguments: &'a HashMap<String, ReplacementValue>, files: &'a LoadedFiles) -> Self {
        Self {
            cache: RefCell::new(HashMap::new()),
            errors: RefCell::new(Vec::new()),
            files,
            parser: liquid::ParserBuilder::with_stdlib().build().unwrap(),
            arguments,
        }
    }

    fn resolve_value(&self, key: &str) -> Option<String> {
        let replacement = self.arguments.get(key);
        match replacement {
            Some(r) => match r {
                ReplacementValue::String(s) => Some(s.into()),
                ReplacementValue::Object(replacement_object) => match replacement_object {
                    ReplacementObject::File { file } => match self.files.get(file) {
                        Some(contents) => {
                            if file.ends_with(".liquid") {
                                return self.render_liquid(contents);
                            }
                            Some(contents.into())
                        }

                        None => {
                            self.errors
                                .borrow_mut()
                                .push(format!("File '{}' not found for key '{}'.", file, key));
                            None
                        }
                    },
                    ReplacementObject::Liquid { liquid } => self.render_liquid(liquid),
                },
            },
            // Shouldn't happen.
            None => None,
        }
    }

    fn render_liquid(&self, liquid: &str) -> Option<String> {
        match self.parser.parse(liquid) {
            Ok(template) => match template.render(self) {
                Ok(result) => Some(result),
                Err(e) => {
                    let error_msg = format!("Liquid render error: {}", e);
                    self.errors.borrow_mut().push(error_msg);
                    None
                }
            },
            Err(e) => {
                let error_msg = format!("Liquid parse error: {}", e);
                self.errors.borrow_mut().push(error_msg);
                None
            }
        }
    }

    fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    fn get_errors(&self) -> Vec<String> {
        self.errors.borrow().clone()
    }

    fn ensure_cached(&self, key: &str) {
        {
            if self.cache.borrow().contains_key(key) {
                return;
            }
        }

        // It would be nice to do something like `self.cache.borrow_mut().entry(...).or_insert(...)`
        // and optimize the typical case where it's not in the cache, assuming most key are only used once in a template.
        // This would could use extra memory when not needed, but it also makes using the cache more complicated and more importantly does not work because `resolve_value` eventually recursively calls this method and the cache would already be borrowed.

        // Not cached, resolve and cache it.
        // There should always be a value found, otherwise Liquid rendering will eventually yield an error about an index not being found.
        if let Some(value) = self.resolve_value(key) {
            self.cache
                .borrow_mut()
                .insert(key.to_string(), liquid::model::Value::scalar(value));
        }
    }
}

impl<'a> ObjectView for DynamicArguments<'a> {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        self.arguments.len() as i64
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
        Box::new(
            self.arguments
                .keys()
                .map(|k| liquid::model::KStringCow::from_ref(k.as_str())),
        )
    }

    fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
        Box::new(std::iter::empty())
    }

    fn iter<'k>(
        &'k self,
    ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k> {
        Box::new(std::iter::empty())
    }

    fn contains_key(&self, index: &str) -> bool {
        self.arguments.contains_key(index)
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        // Ensure the value is cached
        self.ensure_cached(index);

        // SAFETY: This is unsafe but works in practice because:
        // 1. The cache is only modified in ensure_cached
        // 2. Once a value is cached, it's never removed
        // 3. The lifetime 's ensures the DynamicArguments outlives the reference
        unsafe {
            let cache_ptr = self.cache.as_ptr();
            (*cache_ptr).get(index).map(|v| v as &dyn ValueView)
        }
    }
}

impl<'a> ValueView for DynamicArguments<'a> {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> liquid::model::DisplayCow<'_> {
        liquid::model::DisplayCow::Owned(Box::new("DynamicArguments".to_string()))
    }

    fn source(&self) -> liquid::model::DisplayCow<'_> {
        self.render()
    }

    fn type_name(&self) -> &'static str {
        "object"
    }

    fn query_state(&self, state: liquid::model::State) -> bool {
        match state {
            liquid::model::State::Truthy => true,
            liquid::model::State::DefaultValue => false,
            liquid::model::State::Empty => self.arguments.is_empty(),
            liquid::model::State::Blank => false,
        }
    }

    fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
        liquid::model::KStringCow::from_ref("DynamicArguments")
    }

    fn to_value(&self) -> liquid::model::Value {
        let cache = self.cache.borrow();
        let mut obj = liquid::object!({});
        for (k, v) in cache.iter() {
            obj.insert(k.clone().into(), v.clone());
        }
        liquid::model::Value::Object(obj)
    }

    fn as_object(&self) -> Option<&dyn ObjectView> {
        Some(self)
    }
}

impl<'a> std::fmt::Debug for DynamicArguments<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicArguments")
            .field("arguments", &self.arguments.len())
            .field("cache", &self.cache.borrow().len())
            .finish()
    }
}

impl ConfigurableString {
    /// Process a ReplacementValue and return the resulting string.
    fn process_replacement_value(
        &self,
        value: &ReplacementValue,
        files: &LoadedFiles,
    ) -> Result<String, String> {
        match value {
            ReplacementValue::String(s) => Ok(s.into()),
            ReplacementValue::Object(obj) => self.process_replacement_object(obj, files),
        }
    }

    /// Process a ReplacementObject (File or Liquid) and return the resulting string.
    fn process_replacement_object(
        &self,
        obj: &ReplacementObject,
        files: &LoadedFiles,
    ) -> Result<String, String> {
        match obj {
            ReplacementObject::File { file } => {
                match files.get(file) {
                    Some(contents) => {
                        if file.ends_with(".liquid") {
                            // File contains a liquid template, render it
                            self.render_liquid_template(contents, files)
                        } else {
                            // Plain file, return contents as-is
                            Ok(contents.clone())
                        }
                    }
                    None => Err(format!("File '{}' not found.", file)),
                }
            }
            ReplacementObject::Liquid { liquid } => self.render_liquid_template(liquid, files),
        }
    }

    /// Render a liquid template string with the arguments as context.
    fn render_liquid_template(
        &self,
        template_str: &str,
        files: &LoadedFiles,
    ) -> Result<String, String> {
        let parser = liquid::ParserBuilder::with_stdlib()
            .build()
            .map_err(|e| format!("Failed to build liquid parser: {}", e))?;

        let template = parser
            .parse(template_str)
            .map_err(|e| format!("Failed to parse template: {}", e))?;

        let empty_context;
        let context = match &self.arguments {
            Some(r) => r,
            None => {
                empty_context = HashMap::new();
                &empty_context
            }
        };
        let dynamic_arguments = DynamicArguments::new(context, files);

        let result = template
            .render(&dynamic_arguments)
            .map_err(|e| format!("Failed to render template: {}", e))?;

        // Check if there were any errors during file loading or liquid rendering
        if dynamic_arguments.has_errors() {
            let errors = dynamic_arguments.get_errors();
            Err(format!(
                "Errors during template processing:\n{}",
                errors.join("\n")
            ))
        } else {
            Ok(result)
        }
    }

    pub fn build(&self, files: &LoadedFiles) -> Result<String, String> {
        self.process_replacement_value(&self.base, files)
    }
}
