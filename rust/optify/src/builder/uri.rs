use percent_encoding::{percent_encode, AsciiSet, CONTROLS};
use std::path::Path;

const URI_SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b'%');

#[cfg(not(target_os = "windows"))]
const FILE_URI_SEGMENT: &AsciiSet = &URI_SEGMENT.add(b'\\');

/// Converts a file path to a file URI string.
/// An alternative to the "url" crate's `Url::from_file_path` method
/// because the crate depends on "kstring" and
/// "kstring@2.0.4 requires rustc 1.96.0" which rb-sys doesn't use yet as of version 0.9.128.
pub(crate) fn path_to_file_uri(path: &Path) -> Result<String, String> {
    let mut result = "file://".to_owned();

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::ffi::OsStrExt;

        for component in path.components().skip(1) {
            result.push('/');
            result.extend(percent_encode(
                component.as_os_str().as_bytes(),
                FILE_URI_SEGMENT,
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::path::{Component, Prefix};

        let mut components = path.components();
        match components.next() {
            Some(Component::Prefix(ref prefix)) => match prefix.kind() {
                Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                    result.push('/');
                    result.push(letter as char);
                    result.push(':');
                }
                _ => {
                    return Err(format!(
                        "Failed to convert schema path to a file URI: {}",
                        path.display()
                    ));
                }
            },
            _ => {
                return Err(format!(
                    "Failed to convert schema path to a file URI: {}",
                    path.display()
                ));
            }
        }

        for component in components {
            if component == Component::RootDir {
                continue;
            }

            let component = component.as_os_str().to_str().ok_or_else(|| {
                format!(
                    "Failed to convert schema path to a UTF-8 file URI: {}",
                    path.display()
                )
            })?;

            result.push('/');
            result.extend(percent_encode(component.as_bytes(), URI_SEGMENT));
        }
    }

    Ok(result)
}
