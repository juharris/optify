use optify::provider::{OptionAnalysis, OptionOrigin, OptionSource};

/// Formats an analysis as human-readable lines.
pub fn format_analysis_text(key_path: &str, analysis: &OptionAnalysis) -> String {
    let mut lines = Vec::new();
    if let Some(blocked_by) = &analysis.blocked_by {
        lines.push(format!(
            "{key_path} is blocked: {} was set to a non-object by {}",
            blocked_by.path,
            format_source(&blocked_by.source)
        ));
        push_sources(&mut lines, "blocked", &blocked_by.overridden);
    }
    for origin in &analysis.origins {
        push_origin(&mut lines, origin);
    }
    lines.join("\n")
}

fn format_source(source: &OptionSource) -> String {
    match source {
        OptionSource::Feature { import_chain } => format!(
            "{} ({})",
            source.defining_feature().unwrap_or_default(),
            import_chain.join(" -> ")
        ),
        OptionSource::Overrides => "overrides".to_owned(),
    }
}

fn push_origin(lines: &mut Vec<String>, origin: &OptionOrigin) {
    lines.push(format!(
        "{}: {}",
        origin.path,
        format_source(&origin.source)
    ));
    push_sources(lines, "overrode", &origin.overridden);
}

fn push_sources(lines: &mut Vec<String>, label: &str, sources: &[OptionSource]) {
    lines.extend(
        sources
            .iter()
            .map(|source| format!("  {label}: {}", format_source(source))),
    );
}
