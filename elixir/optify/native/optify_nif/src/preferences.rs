use optify::provider::GetOptionsPreferences;
use rustler::{NifResult, ResourceArc, Term};
use std::sync::Mutex;

pub struct PreferencesResource(pub Mutex<GetOptionsPreferences>);

#[rustler::resource_impl]
impl rustler::Resource for PreferencesResource {}

#[rustler::nif]
pub fn preferences_new() -> ResourceArc<PreferencesResource> {
    ResourceArc::new(PreferencesResource(
        Mutex::new(GetOptionsPreferences::new()),
    ))
}

#[rustler::nif]
pub fn preferences_are_configurable_strings_enabled(
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<bool> {
    let guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard.are_configurable_strings_enabled)
}

#[rustler::nif]
pub fn preferences_enable_configurable_strings(
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<ResourceArc<PreferencesResource>> {
    let mut guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard.are_configurable_strings_enabled = true;
    drop(guard);
    Ok(preferences)
}

#[rustler::nif]
pub fn preferences_disable_configurable_strings(
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<ResourceArc<PreferencesResource>> {
    let mut guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard.are_configurable_strings_enabled = false;
    drop(guard);
    Ok(preferences)
}

#[rustler::nif]
pub fn preferences_set_constraints_json(
    preferences: ResourceArc<PreferencesResource>,
    constraints_json: Option<String>,
) -> NifResult<ResourceArc<PreferencesResource>> {
    let mut guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard.set_constraints_json(constraints_json.as_deref());
    drop(guard);
    Ok(preferences)
}

#[rustler::nif]
pub fn preferences_get_constraints_json(
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Option<String>> {
    let guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard
        .constraints
        .as_ref()
        .map(|c| serde_json::to_string(&c.constraints).unwrap()))
}

#[rustler::nif]
pub fn preferences_set_overrides_json(
    preferences: ResourceArc<PreferencesResource>,
    overrides_json: Option<String>,
) -> NifResult<ResourceArc<PreferencesResource>> {
    let mut guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard.overrides =
        overrides_json.map(|s| serde_json::from_str(&s).expect("overrides should be valid JSON"));
    drop(guard);
    Ok(preferences)
}

#[rustler::nif]
pub fn preferences_set_overrides<'a>(
    preferences: ResourceArc<PreferencesResource>,
    overrides: Option<Term<'a>>,
) -> NifResult<ResourceArc<PreferencesResource>> {
    let mut guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard.overrides = match overrides {
        Some(term) => Some(crate::term_to_json_value(term)?),
        None => None,
    };
    drop(guard);
    Ok(preferences)
}

#[rustler::nif]
pub fn preferences_get_overrides_json(
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<Option<String>> {
    let guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard
        .overrides
        .as_ref()
        .map(|o| serde_json::to_string(&o).unwrap()))
}

#[rustler::nif]
pub fn preferences_set_skip_feature_name_conversion(
    preferences: ResourceArc<PreferencesResource>,
    value: bool,
) -> NifResult<ResourceArc<PreferencesResource>> {
    let mut guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    guard.skip_feature_name_conversion = value;
    drop(guard);
    Ok(preferences)
}

#[rustler::nif]
pub fn preferences_get_skip_feature_name_conversion(
    preferences: ResourceArc<PreferencesResource>,
) -> NifResult<bool> {
    let guard = preferences
        .0
        .lock()
        .map_err(|e| rustler::Error::Term(Box::new(format!("Lock poisoned: {}", e))))?;
    Ok(guard.skip_feature_name_conversion)
}
