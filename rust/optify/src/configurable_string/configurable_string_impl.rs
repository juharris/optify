use liquid::ObjectView;
use liquid::ValueView;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ConfigurableString {
    pub template: String,
    pub replacements: HashMap<String, ReplacementValue>,
}

// Dynamic object that resolves values on demand
struct DynamicReplacements<'a> {
    replacements: &'a HashMap<String, ReplacementValue>,
    cache: RefCell<HashMap<String, liquid::model::Value>>,
    parser: liquid::Parser,
    root_path: Option<&'a Path>,
    errors: RefCell<Vec<String>>,
}

impl<'a> DynamicReplacements<'a> {
    fn new(
        replacements: &'a HashMap<String, ReplacementValue>,
        root_path: Option<&'a Path>,
    ) -> Self {
        Self {
            replacements,
            cache: RefCell::new(HashMap::new()),
            parser: liquid::ParserBuilder::with_stdlib().build().unwrap(),
            root_path,
            errors: RefCell::new(Vec::new()),
        }
    }

    fn resolve_value(&self, key: &str) -> Option<String> {
        self.replacements.get(key).map(|value| {
            match value {
                ReplacementValue::String(s) => s.clone(),
                ReplacementValue::Object(obj) => {
                    match obj {
                        ReplacementObject::File { file: file_path } => {
                            // Resolve file path relative to root_path if provided
                            let full_path = if let Some(root) = self.root_path {
                                root.join(file_path)
                            } else {
                                PathBuf::from(file_path)
                            };

                            // Try to read the file content
                            match fs::read_to_string(&full_path) {
                                Ok(content) => content,
                                Err(e) => {
                                    let error_msg = format!(
                                        "Failed to read file '{}': {}",
                                        full_path.display(),
                                        e
                                    );
                                    self.errors.borrow_mut().push(error_msg.clone());
                                    format!("[file error: {}]", e)
                                }
                            }
                        }
                        ReplacementObject::Liquid {
                            liquid: liquid_template,
                        } => {
                            // Render the liquid template with current context
                            match self.parser.parse(liquid_template) {
                                Ok(template) => {
                                    // Use self as the context for nested liquid templates
                                    template.render(self).unwrap_or_else(|e| {
                                        let error_msg = format!("Liquid render error: {}", e);
                                        self.errors.borrow_mut().push(error_msg);
                                        format!("[liquid error: {}]", e)
                                    })
                                }
                                Err(e) => {
                                    let error_msg = format!("Liquid parse error: {}", e);
                                    self.errors.borrow_mut().push(error_msg);
                                    format!("[liquid parse error: {}]", e)
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    fn get_errors(&self) -> Vec<String> {
        self.errors.borrow().clone()
    }

    fn ensure_cached(&self, key: &str) {
        // Check if already cached first
        {
            if self.cache.borrow().contains_key(key) {
                return;
            }
        }

        // Not cached, resolve and cache it
        if let Some(value) = self.resolve_value(key) {
            self.cache
                .borrow_mut()
                .insert(key.to_string(), liquid::model::Value::scalar(value));
        }
    }
}

impl<'a> ObjectView for DynamicReplacements<'a> {
    fn as_value(&self) -> &dyn ValueView {
        self
    }

    fn size(&self) -> i64 {
        self.replacements.len() as i64
    }

    fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
        Box::new(
            self.replacements
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
        // Check cache first
        {
            if self.cache.borrow().contains_key(index) {
                return true;
            }
        } // Release borrow before next check
        self.replacements.contains_key(index)
    }

    fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
        // Ensure the value is cached
        self.ensure_cached(index);

        // SAFETY: This is unsafe but works in practice because:
        // 1. The cache is only modified in ensure_cached
        // 2. Once a value is cached, it's never removed
        // 3. The lifetime 's ensures the DynamicReplacements outlives the reference
        unsafe {
            let cache_ptr = self.cache.as_ptr();
            (*cache_ptr).get(index).map(|v| v as &dyn ValueView)
        }
    }
}

impl<'a> ValueView for DynamicReplacements<'a> {
    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn render(&self) -> liquid::model::DisplayCow<'_> {
        liquid::model::DisplayCow::Owned(Box::new("DynamicReplacements".to_string()))
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
            liquid::model::State::Empty => self.replacements.is_empty(),
            liquid::model::State::Blank => false,
        }
    }

    fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
        liquid::model::KStringCow::from_ref("DynamicReplacements")
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

impl<'a> std::fmt::Debug for DynamicReplacements<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynamicReplacements")
            .field("replacements", &self.replacements.len())
            .field("cache", &self.cache.borrow().len())
            .finish()
    }
}

impl ConfigurableString {
    pub fn build(&self, root_path: Option<&Path>) -> Result<String, String> {
        // Create a liquid parser
        let parser = liquid::ParserBuilder::with_stdlib()
            .build()
            .map_err(|e| format!("Failed to build liquid parser: {}", e))?;

        // Create dynamic replacements object
        let dynamic_replacements = DynamicReplacements::new(&self.replacements, root_path);

        // Parse and render the main template
        let template = parser
            .parse(&self.template)
            .map_err(|e| format!("Failed to parse template: {}", e))?;

        let result = template
            .render(&dynamic_replacements)
            .map_err(|e| format!("Failed to render template: {}", e))?;

        // Check if there were any errors during file loading or liquid rendering
        if dynamic_replacements.has_errors() {
            let errors = dynamic_replacements.get_errors();
            Err(format!(
                "Errors during template processing:\n{}",
                errors.join("\n")
            ))
        } else {
            Ok(result)
        }
    }
}
