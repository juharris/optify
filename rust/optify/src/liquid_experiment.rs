use liquid::object;
use liquid::ObjectView;
use liquid::ValueView;
use std::cell::RefCell;
use std::collections::HashMap;

pub fn experiment_with_liquid() {
    println!("Starting liquid experiments...\n");

    // Try implementing a custom ObjectView with interior mutability
    custom_object_view_with_cache();
}

fn custom_object_view_with_cache() {
    println!("=== Custom ObjectView with Cached Values ===\n");

    // Create a custom type that implements ObjectView with interior mutability
    struct DynamicObject {
        // Cache resolved values so that we can return a reference to the value.
        // Use RefCell to allow interior mutability because of the signature for `get` in the trait.
        cache: RefCell<HashMap<String, liquid::model::Value>>,
    }

    impl DynamicObject {
        fn new() -> Self {
            Self {
                cache: RefCell::new(HashMap::new()),
            }
        }

        // This is where we'd fetch the value dynamically
        fn resolve_value(&self, key: &str) -> Option<String> {
            println!("  [Resolving] {}", key);
            match key {
                "custom_var1" => Some("Value from custom hashmap!".to_string()),
                "custom_var2" => Some("Another custom value".to_string()),
                "user_name" => Some("Alice".to_string()),
                "timestamp" => Some("2024-01-15T10:00:00Z".to_string()),
                _ => {
                    println!("    -> Not found in resolver");
                    None
                }
            }
        }

        // Ensure a value is in the cache
        fn ensure_cached(&self, key: &str) {
            let mut cache = self.cache.borrow_mut();
            if !cache.contains_key(key) {
                if let Some(value) = self.resolve_value(key) {
                    println!("    -> Cached: {}", value);
                    cache.insert(key.to_string(), liquid::model::Value::scalar(value));
                }
            }
        }
    }

    // Implement ObjectView for our custom type
    impl ObjectView for DynamicObject {
        fn as_value(&self) -> &dyn ValueView {
            println!("  [as_value]");
            self
        }

        fn size(&self) -> i64 {
            println!("  [size]");
            // Return 0 since we don't know the size until we resolve everything
            0
        }

        fn keys<'k>(&'k self) -> Box<dyn Iterator<Item = liquid::model::KStringCow<'k>> + 'k> {
            println!("  [keys]");
            // Return empty iterator since we don't know all keys upfront
            Box::new(std::iter::empty())
        }

        fn values<'k>(&'k self) -> Box<dyn Iterator<Item = &'k dyn ValueView> + 'k> {
            println!("  [values]");
            Box::new(std::iter::empty())
        }

        fn iter<'k>(
            &'k self,
        ) -> Box<dyn Iterator<Item = (liquid::model::KStringCow<'k>, &'k dyn ValueView)> + 'k>
        {
            println!("  [iter]");
            Box::new(std::iter::empty())
        }

        fn contains_key(&self, index: &str) -> bool {
            println!("  [contains_key] {}", index);
            // First check cache
            if self.cache.borrow().contains_key(index) {
                return true;
            }
            // Then try to resolve it
            self.resolve_value(index).is_some()
        }

        fn get<'s>(&'s self, index: &str) -> Option<&'s dyn ValueView> {
            println!("  [get] {}", index);

            // Ensure the value is cached
            self.ensure_cached(index);

            // SAFETY: This is unsafe but works in practice because:
            // 1. The cache is only modified in ensure_cached
            // 2. Once a value is cached, it's never removed
            // 3. The lifetime 's ensures the DynamicObject outlives the reference
            unsafe {
                let cache_ptr = self.cache.as_ptr();
                (*cache_ptr).get(index).map(|v| v as &dyn ValueView)
            }
        }
    }

    // Implement ValueView for our custom type
    impl ValueView for DynamicObject {
        fn as_debug(&self) -> &dyn std::fmt::Debug {
            self
        }

        fn render(&self) -> liquid::model::DisplayCow<'_> {
            liquid::model::DisplayCow::Owned(Box::new("DynamicObject".to_string()))
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
                liquid::model::State::Empty => false,
                liquid::model::State::Blank => false,
            }
        }

        fn to_kstr(&self) -> liquid::model::KStringCow<'_> {
            liquid::model::KStringCow::from_ref("DynamicObject")
        }

        fn to_value(&self) -> liquid::model::Value {
            println!("  [ValueView::to_value]");
            // Convert cached values to a regular object
            let cache = self.cache.borrow();
            let mut obj = object!({});
            for (k, v) in cache.iter() {
                obj.insert(k.clone().into(), v.clone());
            }
            liquid::model::Value::Object(obj)
        }

        fn as_object(&self) -> Option<&dyn ObjectView> {
            Some(self)
        }
    }

    impl std::fmt::Debug for DynamicObject {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("DynamicObject")
                .field("cache", &self.cache.borrow().len())
                .finish()
        }
    }

    // Now let's test it
    let dynamic_obj = DynamicObject::new();

    let template = r#"
Custom variable 1: {{ custom_var1 }}
Custom variable 2: {{ custom_var2 }}
User name: {{ user_name }}
Timestamp: {{ timestamp }}
"#;

    let parser = liquid::ParserBuilder::with_stdlib()
        .build()
        .expect("Failed to build parser");

    let liquid_template = parser.parse(template).expect("Failed to parse template");

    println!("\nRendering template:\n");
    match liquid_template.render(&dynamic_obj) {
        Ok(result) => {
            println!("\n=== Rendered Output ===");
            println!("{}", result);
        }
        Err(e) => {
            println!("\nError rendering: {}", e);
            println!("This is expected for undefined variables like 'missing_var'");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquid_experiments() {
        experiment_with_liquid();
    }
}
