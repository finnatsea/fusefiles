//! File extension to language mapping for syntax highlighting

use std::collections::HashMap;

/// Get the mapping of file extensions to language names
pub fn get_language_map() -> HashMap<&'static str, &'static str> {
    [
        ("py", "python"),
        ("c", "c"),
        ("cpp", "cpp"),
        ("java", "java"),
        ("js", "javascript"),
        ("ts", "typescript"),
        ("html", "html"),
        ("css", "css"),
        ("xml", "xml"),
        ("json", "json"),
        ("yaml", "yaml"),
        ("yml", "yaml"),
        ("sh", "bash"),
        ("rb", "ruby"),
    ].iter().cloned().collect()
}

/// Get the language name for a given file extension
pub fn get_language_for_extension(extension: &str) -> &str {
    get_language_map().get(extension).unwrap_or(&"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_extensions() {
        assert_eq!(get_language_for_extension("py"), "python");
        assert_eq!(get_language_for_extension("js"), "javascript");
        assert_eq!(get_language_for_extension("rs"), ""); // Not in the map
    }

    #[test]
    fn test_yaml_extensions() {
        assert_eq!(get_language_for_extension("yaml"), "yaml");
        assert_eq!(get_language_for_extension("yml"), "yaml");
    }

    #[test]
    fn test_unknown_extension() {
        assert_eq!(get_language_for_extension("unknown"), "");
        assert_eq!(get_language_for_extension(""), "");
    }
}