use rustler::{Env, Term};

pub mod preferences;
pub mod provider;
pub mod provider_builder;
pub mod watcher;
pub mod watcher_builder;

pub fn json_value_to_term<'a>(env: Env<'a>, value: &serde_json::Value) -> Term<'a> {
    rustler::serde::to_term(env, value).expect("Failed to convert JSON value to Elixir term")
}

pub fn term_to_json_value(term: Term) -> Result<serde_json::Value, rustler::Error> {
    rustler::serde::from_term(term)
        .map_err(|e| rustler::Error::Term(Box::new(format!("Failed to decode term: {}", e))))
}

rustler::init!("Elixir.Optify.Native");
