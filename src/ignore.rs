//! Custom ignore pattern handling built around glob matching.
//!
//! This module focuses on user-specified `--ignore` patterns. Gitignore semantics
//! are handled through the `ignore` crate in the traversal code, which means this
//! helper only needs to reason about additional patterns supplied via CLI flags.

use crate::{FilesToPromptError, Result};
use glob::Pattern;
use std::path::Path;

/// Normalise a path to a forward-slash separated string for glob matching.
fn normalise_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[derive(Clone)]
struct CustomPattern {
    original: String,
    glob: Pattern,
    directory_only: bool,
}

/// Represents user-supplied ignore patterns.
#[derive(Clone)]
pub struct CustomIgnore {
    patterns: Vec<CustomPattern>,
    ignore_files_only: bool,
}

impl CustomIgnore {
    /// Build the matcher from raw pattern strings.
    pub fn new(patterns: Vec<String>, ignore_files_only: bool) -> Result<Self> {
        let mut compiled = Vec::new();
        for pattern in patterns {
            let trimmed = pattern.trim();
            if trimmed.is_empty() {
                continue;
            }
            let glob = Pattern::new(trimmed)
                .map_err(|e| FilesToPromptError::PatternError(e.msg.into()))?;
            compiled.push(CustomPattern {
                original: trimmed.to_string(),
                glob,
                directory_only: trimmed.ends_with('/'),
            });
        }

        Ok(Self {
            patterns: compiled,
            ignore_files_only,
        })
    }

    /// Returns true when no patterns were provided.
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Exposes the `ignore-files-only` flag.
    pub fn ignore_files_only(&self) -> bool {
        self.ignore_files_only
    }

    /// Should the given file be ignored?
    pub fn should_ignore_file(&self, path: &Path) -> bool {
        self.should_ignore(path, true)
    }

    /// Should the given directory be ignored?
    pub fn should_ignore_dir(&self, path: &Path) -> bool {
        self.should_ignore(path, false)
    }

    fn should_ignore(&self, path: &Path, is_file: bool) -> bool {
        if self.patterns.is_empty() {
            return false;
        }

        if !is_file && self.ignore_files_only {
            return false;
        }

        self.patterns
            .iter()
            .any(|pattern| Self::matches_pattern(pattern, path, is_file))
    }

    fn matches_pattern(pattern: &CustomPattern, path: &Path, is_file: bool) -> bool {
        let glob = &pattern.glob;
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if glob.matches(name) {
                return true;
            }
            if !is_file {
                let with_slash = format!("{}/", name);
                if glob.matches(&with_slash) {
                    return true;
                }
            }
        }

        let normalised = normalise_path(path);
        if glob.matches(&normalised) {
            return true;
        }

        if !is_file {
            let mut with_trailing = normalised.clone();
            if !with_trailing.ends_with('/') {
                with_trailing.push('/');
            }
            if glob.matches(&with_trailing) {
                return true;
            }

            if pattern.directory_only {
                let target = pattern.original.trim_end_matches('/');
                if normalised == target || normalised.starts_with(&format!("{}/", target)) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn path(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn empty_patterns_never_ignore() {
        let matcher = CustomIgnore::new(vec![], false).unwrap();
        assert!(!matcher.should_ignore_file(&path("foo")));
        assert!(!matcher.should_ignore_dir(&path("foo")));
    }

    #[test]
    fn ignores_files_using_globs() {
        let matcher = CustomIgnore::new(vec!["*.log".into(), "temp*".into()], false).unwrap();
        assert!(matcher.should_ignore_file(&path("debug.log")));
        assert!(matcher.should_ignore_file(&path("temp_data.txt")));
        assert!(!matcher.should_ignore_file(&path("keep.txt")));
    }

    #[test]
    fn ignores_directories_when_allowed() {
        let matcher = CustomIgnore::new(vec!["build/".into()], false).unwrap();
        assert!(matcher.should_ignore_dir(&path("build")));
        assert!(matcher.should_ignore_dir(&path("build/subdir")));
    }

    #[test]
    fn ignore_files_only_skips_directories() {
        let matcher = CustomIgnore::new(vec!["build/".into()], true).unwrap();
        assert!(!matcher.should_ignore_dir(&path("build")));
        assert!(matcher.ignore_files_only());
    }

    #[test]
    fn matches_against_normalised_paths() {
        let matcher =
            CustomIgnore::new(vec!["src/**/*.rs".into(), "nested/file.txt".into()], false).unwrap();
        assert!(matcher.should_ignore_file(&path("src/lib.rs")));
        assert!(matcher.should_ignore_file(&path("src/foo/mod.rs")));
        assert!(matcher.should_ignore_file(&path("nested/file.txt")));
        assert!(!matcher.should_ignore_file(&path("nested/file.md")));
    }
}
