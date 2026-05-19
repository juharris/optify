use std::path::Path;

pub(super) fn get_canonical_feature_name(path: &Path, directory: &Path) -> String {
    path.strip_prefix(directory)
        .unwrap()
        .with_extension("")
        .to_str()
        .expect("path should be valid Unicode")
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_canonical_feature_name() {
        let directory = std::path::Path::new("wtv");
        let path = directory.join("dir1").join("dir2").join("feature_B.json");
        assert_eq!(
            "dir1/dir2/feature_B",
            get_canonical_feature_name(&path, directory)
        );
    }
}
