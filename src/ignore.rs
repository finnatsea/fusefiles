//! Gitignore file parsing and pattern matching logic

use crate::{FilesToPromptError, Result};
use glob::Pattern;
use std::fs;
use std::path::Path;

/// Handles ignore patterns from gitignore files and custom patterns
pub struct IgnoreChecker {
    gitignore_patterns: Vec<Pattern>,
    custom_patterns: Vec<Pattern>,
    ignore_files_only: bool,
}

impl IgnoreChecker {
    /// Create a new IgnoreChecker
    pub fn new(ignore_files_only: bool) -> Self {
        Self {
            gitignore_patterns: Vec::new(),
            custom_patterns: Vec::new(),
            ignore_files_only,
        }
    }

    /// Add patterns from a .gitignore file
    pub fn add_gitignore_file(&mut self, gitignore_path: &Path) -> Result<()> {
        if gitignore_path.exists() && gitignore_path.is_file() {
            let content = fs::read_to_string(gitignore_path)?;
            for line in content.lines() {
                let line = line.trim();
                // Skip empty lines and comments
                if !line.is_empty() && !line.starts_with('#') {
                    match Pattern::new(line) {
                        Ok(pattern) => self.gitignore_patterns.push(pattern),
                        Err(e) => return Err(FilesToPromptError::PatternError(e.to_string())),
                    }
                }
            }
        }
        Ok(())
    }

    /// Add custom ignore patterns
    pub fn add_custom_patterns(&mut self, patterns: &[String]) -> Result<()> {
        for pattern_str in patterns {
            if !pattern_str.is_empty() {
                match Pattern::new(pattern_str) {
                    Ok(pattern) => self.custom_patterns.push(pattern),
                    Err(e) => return Err(FilesToPromptError::PatternError(e.to_string())),
                }
            }
        }
        Ok(())
    }

    /// Check if a path should be ignored based on gitignore rules
    pub fn should_ignore_gitignore(&self, path: &Path) -> bool {
        Self::matches_any_pattern(path, &self.gitignore_patterns)
    }

    /// Check if a path should be ignored based on custom patterns
    pub fn should_ignore_custom(&self, path: &Path, is_file: bool) -> bool {
        // If ignore_files_only is true and this is a directory, don't ignore
        if self.ignore_files_only && !is_file {
            return false;
        }
        Self::matches_any_pattern(path, &self.custom_patterns)
    }

    /// Check if a path matches any of the given patterns
    fn matches_any_pattern(path: &Path, patterns: &[Pattern]) -> bool {
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Check against filename
            if patterns.iter().any(|pattern| pattern.matches(filename)) {
                return true;
            }

            // For directories, also check with trailing slash
            if path.is_dir() {
                let dir_pattern = format!("{}/", filename);
                if patterns.iter().any(|pattern| pattern.matches(&dir_pattern)) {
                    return true;
                }
            }
        }
        false
    }
}

/// Read gitignore patterns from a directory
pub fn read_gitignore_patterns(dir_path: &Path) -> Result<Vec<String>> {
    let gitignore_path = dir_path.join(".gitignore");
    if gitignore_path.exists() && gitignore_path.is_file() {
        let content = fs::read_to_string(gitignore_path)?;
        Ok(content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect())
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_empty_checker() {
        let checker = IgnoreChecker::new(false);
        let path = PathBuf::from("test.txt");
        assert!(!checker.should_ignore_gitignore(&path));
        assert!(!checker.should_ignore_custom(&path, true));
    }

    #[test]
    fn test_custom_patterns() {
        let mut checker = IgnoreChecker::new(false);
        checker
            .add_custom_patterns(&["*.log".to_string(), "temp*".to_string()])
            .unwrap();

        assert!(checker.should_ignore_custom(&PathBuf::from("test.log"), true));
        assert!(checker.should_ignore_custom(&PathBuf::from("temp_file"), true));
        assert!(!checker.should_ignore_custom(&PathBuf::from("test.txt"), true));
    }

    #[test]
    fn test_ignore_files_only() {
        let mut checker = IgnoreChecker::new(true);
        checker.add_custom_patterns(&["test*".to_string()]).unwrap();

        // Should ignore files matching pattern
        assert!(checker.should_ignore_custom(&PathBuf::from("test.txt"), true));
        // Should NOT ignore directories when ignore_files_only is true
        assert!(!checker.should_ignore_custom(&PathBuf::from("test_dir"), false));
    }

    #[test]
    fn test_pattern_matching() {
        let patterns = vec![
            Pattern::new("*.txt").unwrap(),
            Pattern::new("temp*").unwrap(),
        ];

        assert!(IgnoreChecker::matches_any_pattern(
            &PathBuf::from("test.txt"),
            &patterns
        ));
        assert!(IgnoreChecker::matches_any_pattern(
            &PathBuf::from("temp_file"),
            &patterns
        ));
        assert!(!IgnoreChecker::matches_any_pattern(
            &PathBuf::from("test.py"),
            &patterns
        ));
    }
}
