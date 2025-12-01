#![deny(clippy::all)]

use napi::Env;

pub fn convert_to_js_object(env: Env, value: &serde_json::Value) -> napi::JsUnknown {
  match value {
    serde_json::Value::Null => env.get_null().unwrap().into_unknown(),
    serde_json::Value::Bool(b) => env.get_boolean(*b).unwrap().into_unknown(),
    serde_json::Value::Number(n) => {
      if let Some(i) = n.as_i64() {
        env.create_int64(i).unwrap().into_unknown()
      } else if let Some(f) = n.as_f64() {
        env.create_double(f).unwrap().into_unknown()
      } else {
        env.get_null().unwrap().into_unknown()
      }
    }
    serde_json::Value::String(s) => env.create_string(s).unwrap().into_unknown(),
    serde_json::Value::Array(arr) => {
      let mut js_array = env.create_array_with_length(arr.len()).unwrap();
      for (i, item) in arr.iter().enumerate() {
        js_array
          .set_element(i as u32, convert_to_js_object(env, item))
          .unwrap();
      }
      js_array.into_unknown()
    }
    serde_json::Value::Object(map) => {
      let mut js_obj = env.create_object().unwrap();
      for (key, val) in map.iter() {
        js_obj
          .set(key.as_str(), convert_to_js_object(env, val))
          .unwrap();
      }
      js_obj.into_unknown()
    }
  }
}
