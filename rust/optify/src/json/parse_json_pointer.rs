/// Parses a JSON pointer (RFC 6901) into unescaped segments.
///
/// The empty pointer for the root is not supported because callers need a path to a specific value.
pub(crate) fn parse_json_pointer(pointer: &str) -> Result<Vec<String>, String> {
    let Some(rest) = pointer.strip_prefix('/') else {
        return Err(format!(
            "Invalid key path {pointer:?}: it must be a JSON pointer that starts with \"/\", such as \"/myConfig/myKey\"."
        ));
    };

    rest.split('/')
        .map(|segment| unescape_segment(segment, pointer))
        .collect()
}

fn unescape_segment(segment: &str, pointer: &str) -> Result<String, String> {
    let mut result = String::with_capacity(segment.len());
    let mut characters = segment.chars();
    while let Some(character) = characters.next() {
        if character == '~' {
            match characters.next() {
                Some('0') => result.push('~'),
                Some('1') => result.push('/'),
                _ => {
                    return Err(format!(
                        "Invalid key path {pointer:?}: \"~\" must be escaped as \"~0\" and \"/\" must be escaped as \"~1\"."
                    ))
                }
            }
        } else {
            result.push(character);
        }
    }
    Ok(result)
}
