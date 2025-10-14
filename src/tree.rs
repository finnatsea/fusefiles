//! Tree generation for directory structure visualization

use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use walkdir::WalkDir;
use crate::{Result, TocMode};
use crate::ignore::IgnoreChecker;

/// Represents a node in the directory tree
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_file: bool,
    pub children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    pub fn new(name: String, path: PathBuf, is_file: bool) -> Self {
        Self {
            name,
            path,
            is_file,
            children: BTreeMap::new(),
        }
    }

    /// Add a child node to this node
    pub fn add_child(&mut self, child: TreeNode) {
        self.children.insert(child.name.clone(), child);
    }

    /// Get the total number of nodes in this tree (including self)
    pub fn count_nodes(&self) -> usize {
        1 + self.children.values().map(|child| child.count_nodes()).sum::<usize>()
    }

    /// Get the number of file nodes in this tree
    pub fn count_files(&self) -> usize {
        let self_count = if self.is_file { 1 } else { 0 };
        self_count + self.children.values().map(|child| child.count_files()).sum::<usize>()
    }

    /// Estimate the number of lines this tree would take to render
    pub fn estimate_render_lines(&self, show_files: bool) -> usize {
        if !show_files && self.is_file {
            return 0;
        }
        
        1 + self.children.values()
            .map(|child| child.estimate_render_lines(show_files))
            .sum::<usize>()
    }
}

/// Handles tree generation for directory structures
pub struct TreeGenerator {
    extensions: Vec<String>,
    include_hidden: bool,
    ignore_files_only: bool,
    ignore_gitignore: bool,
    ignore_patterns: Vec<String>,
}

impl TreeGenerator {
    pub fn new(
        extensions: Vec<String>,
        include_hidden: bool,
        ignore_files_only: bool,
        ignore_gitignore: bool,
        ignore_patterns: Vec<String>,
    ) -> Self {
        Self {
            extensions,
            include_hidden,
            ignore_files_only,
            ignore_gitignore,
            ignore_patterns,
        }
    }

    /// Generate a tree structure for the given paths
    pub fn generate_tree(&self, paths: &[PathBuf]) -> Result<Vec<TreeNode>> {
        let mut trees = Vec::new();

        for path in paths {
            if path.is_file() {
                if self.should_include_file(path) {
                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_string();
                    trees.push(TreeNode::new(name, path.clone(), true));
                }
            } else if path.is_dir() {
                if let Some(tree) = self.generate_directory_tree(path)? {
                    trees.push(tree);
                }
            }
        }

        Ok(trees)
    }

    /// Generate tree for a single directory
    fn generate_directory_tree(&self, dir_path: &Path) -> Result<Option<TreeNode>> {
        let mut ignore_checker = IgnoreChecker::new(self.ignore_files_only);
        
        // Add custom ignore patterns
        ignore_checker.add_custom_patterns(&self.ignore_patterns)?;

        // Add gitignore patterns if not ignoring them
        if !self.ignore_gitignore {
            ignore_checker.add_gitignore_file(&dir_path.join(".gitignore"))?;
        }

        let dir_name = dir_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        
        let mut root = TreeNode::new(dir_name, dir_path.to_path_buf(), false);

        // Walk directory and build tree
        let walker = WalkDir::new(dir_path)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                // Filter out hidden directories if not including hidden files
                if !self.include_hidden && self.is_hidden_path(e.path()) {
                    return false;
                }
                true
            });

        for entry in walker {
            let entry = entry?;
            let entry_path = entry.path();

            #[cfg(test)]
            println!("Processing path: {:?}", entry_path);

            // Skip the root directory itself
            if entry_path == dir_path {
                #[cfg(test)]
                println!("Skipping root directory");
                continue;
            }

            // Check if this path should be ignored
            if self.should_ignore_path(entry_path, &ignore_checker) {
                #[cfg(test)]
                println!("Path ignored: {:?}", entry_path);
                continue;
            }

            // For files, check if they meet our criteria
            if entry_path.is_file() && !self.should_include_file(entry_path) {
                #[cfg(test)]
                println!("File criteria not met: {:?}", entry_path);
                continue;
            }

            #[cfg(test)]
            println!("Adding to tree: {:?} (is_file: {})", entry_path, entry_path.is_file());

            // Add to tree
            self.add_path_to_tree(&mut root, dir_path, entry_path, entry_path.is_file());
        }

        // Always return the root, even if empty, so tests can see the structure
        Ok(Some(root))
    }

    /// Add a path to the tree structure
    fn add_path_to_tree(&self, root: &mut TreeNode, base_path: &Path, full_path: &Path, is_file: bool) {
        // Get relative path from base
        let relative_path = match full_path.strip_prefix(base_path) {
            Ok(rel) => rel,
            Err(_) => return,
        };

        let mut current = root;
        let components: Vec<_> = relative_path.components().collect();

        for (i, component) in components.iter().enumerate() {
            let name = component.as_os_str().to_str().unwrap_or("?").to_string();
            let is_last = i == components.len() - 1;
            let node_is_file = is_last && is_file;

            if !current.children.contains_key(&name) {
                let node_path = base_path.join(relative_path.iter().take(i + 1).collect::<PathBuf>());
                let node = TreeNode::new(name.clone(), node_path, node_is_file);
                current.children.insert(name.clone(), node);
            }

            current = current.children.get_mut(&name).unwrap();
        }
    }

    /// Check if a path should be ignored
    fn should_ignore_path(&self, path: &Path, ignore_checker: &IgnoreChecker) -> bool {
        // Check if file is hidden and should be excluded
        if !self.include_hidden && self.is_hidden_file(path) {
            return true;
        }

        // Check gitignore rules if not ignoring them
        if !self.ignore_gitignore && ignore_checker.should_ignore_gitignore(path) {
            return true;
        }

        // Check custom ignore patterns
        if ignore_checker.should_ignore_custom(path, path.is_file()) {
            return true;
        }

        false
    }

    /// Check if a file should be included based on extension filters
    fn should_include_file(&self, path: &Path) -> bool {
        if self.extensions.is_empty() {
            return true;
        }

        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            self.extensions.iter().any(|ext| {
                let ext = ext.strip_prefix('.').unwrap_or(ext);
                extension == ext
            })
        } else {
            false
        }
    }

    /// Check if a file is hidden (starts with '.')
    fn is_hidden_file(&self, path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    }

    /// Check if any component of a path is hidden
    fn is_hidden_path(&self, path: &Path) -> bool {
        path.components().any(|component| {
            if let Some(name) = component.as_os_str().to_str() {
                name.starts_with('.') && name != "." && name != ".."
            } else {
                false
            }
        })
    }

    /// Render tree to string format
    pub fn render_tree(&self, trees: &[TreeNode], mode: TocMode) -> String {
        if trees.is_empty() {
            return String::new();
        }

        let mut output = Vec::new();

        // Determine whether to show files based on mode and auto-detection
        let show_files = match mode {
            TocMode::DirsOnly => false,
            TocMode::FilesAndDirs => true,
            TocMode::Auto => {
                // Estimate total lines with files
                let total_lines: usize = trees.iter()
                    .map(|tree| tree.estimate_render_lines(true))
                    .sum();
                total_lines < 100
            }
        };

        for (i, tree) in trees.iter().enumerate() {
            let is_last = i == trees.len() - 1;
            self.render_node(tree, "", is_last, show_files, &mut output);
        }

        output.join("\n")
    }

    /// Render a single tree node with proper indentation and tree characters
    fn render_node(
        &self,
        node: &TreeNode,
        prefix: &str,
        is_last: bool,
        show_files: bool,
        output: &mut Vec<String>,
    ) {
        // Skip files if we're not showing them
        if !show_files && node.is_file {
            return;
        }

        // Choose the appropriate tree character
        let connector = if is_last { "└── " } else { "├── " };
        
        // Add file/directory indicator
        let name = if node.is_file {
            node.name.clone()
        } else {
            format!("{}/", node.name)
        };

        output.push(format!("{}{}{}", prefix, connector, name));

        // Render children
        let children: Vec<_> = node.children.values().collect();
        for (i, child) in children.iter().enumerate() {
            let child_is_last = i == children.len() - 1;
            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            
            self.render_node(child, &child_prefix, child_is_last, show_files, output);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new(
            "test".to_string(),
            PathBuf::from("/test"),
            false
        );
        
        assert_eq!(node.name, "test");
        assert_eq!(node.path, PathBuf::from("/test"));
        assert!(!node.is_file);
        assert!(node.children.is_empty());
    }

    #[test]
    fn test_tree_node_count() {
        let mut root = TreeNode::new("root".to_string(), PathBuf::from("/root"), false);
        let child1 = TreeNode::new("child1".to_string(), PathBuf::from("/root/child1"), true);
        let child2 = TreeNode::new("child2".to_string(), PathBuf::from("/root/child2"), false);
        
        root.add_child(child1);
        root.add_child(child2);
        
        assert_eq!(root.count_nodes(), 3);
        assert_eq!(root.count_files(), 1);
    }

    #[test]
    fn test_basic_tree_generation() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Create test structure
        fs::create_dir_all(base_path.join("subdir")).unwrap();
        fs::write(base_path.join("file1.txt"), "content1").unwrap();
        fs::write(base_path.join("subdir/file2.txt"), "content2").unwrap();
        
        let generator = TreeGenerator::new(
            vec![],
            false,
            false,
            true, // ignore gitignore
            vec![],
        );
        
        let trees = generator.generate_tree(&[base_path.to_path_buf()]).unwrap();
        
        // Debug output
        println!("Generated trees: {}", trees.len());
        for (i, tree) in trees.iter().enumerate() {
            println!("Tree {}: {} (children: {})", i, tree.name, tree.children.len());
        }
        
        assert!(trees.len() >= 1, "Expected at least 1 tree, got {}", trees.len());
        
        let tree = &trees[0];
        assert!(!tree.is_file);
        assert!(tree.children.len() >= 1, "Expected at least 1 child, got {}", tree.children.len());
    }

    #[test]
    fn test_tree_rendering() {
        let mut root = TreeNode::new("root".to_string(), PathBuf::from("/root"), false);
        let mut subdir = TreeNode::new("subdir".to_string(), PathBuf::from("/root/subdir"), false);
        let file1 = TreeNode::new("file1.txt".to_string(), PathBuf::from("/root/file1.txt"), true);
        let file2 = TreeNode::new("file2.txt".to_string(), PathBuf::from("/root/subdir/file2.txt"), true);
        
        subdir.add_child(file2);
        root.add_child(file1);
        root.add_child(subdir);
        
        let generator = TreeGenerator::new(vec![], false, false, true, vec![]);
        let output = generator.render_tree(&[root], TocMode::FilesAndDirs);
        
        assert!(output.contains("root/"));
        assert!(output.contains("├── file1.txt"));
        assert!(output.contains("└── subdir/"));
        assert!(output.contains("    └── file2.txt"));
    }

    #[test]
    fn test_auto_mode_line_estimation() {
        let mut root = TreeNode::new("root".to_string(), PathBuf::from("/root"), false);
        
        // Create many children to exceed 100 lines
        for i in 0..50 {
            let file = TreeNode::new(format!("file{}.txt", i), PathBuf::from(format!("/root/file{}.txt", i)), true);
            root.add_child(file);
        }
        
        assert!(root.estimate_render_lines(true) > 50);
        assert_eq!(root.estimate_render_lines(false), 1); // Only the root directory
    }
}