#![deny(clippy::all)]

use napi::bindgen_prelude::FromNapiValue;

use crate::metadata::JsOptionsMetadata;

/// A feature configuration to add to a builder.
#[napi(object)]
pub struct Feature {
  #[napi(ts_type = "unknown")]
  pub conditions: Option<serde_json::Value>,
  pub imports: Option<Vec<String>>,
  #[napi(ts_type = "unknown")]
  pub metadata: JsOptionsMetadata,
  #[napi(ts_type = "unknown")]
  pub options: serde_json::Value,
  #[napi(ts_type = "Policies")]
  pub policies: Option<serde_json::Value>,
}

/// A feature converted and validated at the JavaScript boundary.
pub struct ParsedFeature(pub optify::Feature);

fn validate_metadata(metadata: &optify::schema::metadata::OptionsMetadata) -> napi::Result<()> {
  if metadata.name.as_deref().is_none_or(str::is_empty) {
    return Err(napi::Error::from_reason(
      "Feature metadata must have a non-empty name",
    ));
  }
  Ok(())
}

/// Deserializes and validates a feature while preserving its JSON condition representation.
#[napi(
  js_name = "_featureFromJson",
  ts_return_type = "Feature & { metadata: OptionsMetadata }"
)]
pub fn feature_from_json(json: String) -> napi::Result<Feature> {
  let value: serde_json::Value = serde_json::from_str(&json)
    .map_err(|error| napi::Error::from_reason(format!("Invalid feature JSON: {error}")))?;
  // ConditionExpression serializes differently from the input syntax accepted by its deserializer.
  let conditions = value
    .get("conditions")
    .filter(|value| !value.is_null())
    .cloned();
  let policies = value
    .get("policies")
    .filter(|value| !value.is_null())
    .cloned();
  let feature: optify::Feature = serde_json::from_value(value)
    .map_err(|error| napi::Error::from_reason(format!("Invalid feature JSON: {error}")))?;
  validate_metadata(&feature.metadata)?;

  Ok(Feature {
    conditions,
    imports: feature.imports,
    metadata: JsOptionsMetadata {
      inner: feature.metadata,
    },
    options: feature.options,
    policies,
  })
}

impl FromNapiValue for ParsedFeature {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    value: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    let feature = unsafe { Feature::from_napi_value(env, value) }?;
    validate_metadata(&feature.metadata.inner)?;
    let conditions = feature
      .conditions
      .map(serde_json::from_value)
      .transpose()
      .map_err(|error| napi::Error::from_reason(format!("Invalid feature conditions: {error}")))?;
    let policies = feature
      .policies
      .map(serde_json::from_value)
      .transpose()
      .map_err(|error| napi::Error::from_reason(format!("Invalid feature policies: {error}")))?;
    Ok(Self(optify::Feature {
      conditions,
      imports: feature.imports,
      metadata: feature.metadata.inner,
      options: feature.options,
      policies,
    }))
  }
}
