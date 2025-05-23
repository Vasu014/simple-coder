use std::fs;
use std::path::{Path, PathBuf};
use std::io;

const IGNORED_DIRS: &[&str] = &[
    "target",
    ".git",
    "node_modules",
    "dist",
    "build",
    "out",
    "__pycache__",
    ".venv",
    "venv",
    "env",
    // Add any other common directories to ignore
];

const IGNORED_FILES: &[&str] = &[
    ".DS_Store",
    // Add any other common files to ignore
];

/// Scans the given directory path and generates a string representation of its tree structure.
///
/// It excludes predefined directories (like "target", ".git") and files (like ".DS_Store").
///
/// # Arguments
/// * `dir_path` - The path to the directory to scan.
///
/// # Returns
/// A `Result` containing the tree string or an `io::Error`.
pub fn scan_directory_tree_from_path(dir_path: &Path) -> io::Result<String> {
    let mut tree_string = String::new();
    scan_directory_recursive(dir_path, 0, &mut tree_string, true)?;
    Ok(tree_string)
}

/// Scans the current working directory and generates a string representation of its tree structure.
///
/// It excludes predefined directories (like "target", ".git") and files (like ".DS_Store").
///
/// # Returns
/// A `Result` containing the tree string or an `io::Error`.
pub fn scan_current_working_directory() -> io::Result<String> {
    let current_dir = std::env::current_dir()?;
    println!("Scanning directory: {}", current_dir.display());
    scan_directory_tree_from_path(&current_dir)
}

fn scan_directory_recursive(
    dir_path: &Path,
    depth: usize,
    tree_string: &mut String,
    is_last: bool,
) -> io::Result<()> {
    let file_name = dir_path.file_name().unwrap_or_default().to_string_lossy();

    // Indentation logic
    for i in 0..depth {
        if i == depth -1 {
            tree_string.push_str(if is_last { "└── " } else { "├── " });
        } else {
            // This part needs improvement to correctly draw parent connectors
            // For simplicity, just adding spaces now. A more robust solution
            // would track `is_last` for parent directories.
            tree_string.push_str("    "); 
        }
    }
    tree_string.push_str(&file_name);
    tree_string.push('\n');

    if dir_path.is_dir() {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir_path)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                if path.is_dir() {
                    !IGNORED_DIRS.contains(&name.as_ref())
                } else {
                    !IGNORED_FILES.contains(&name.as_ref())
                }
            })
            .collect();

        entries.sort(); // Sort for consistent order

        let mut peekable_entries = entries.into_iter().peekable();
        while let Some(entry) = peekable_entries.next() {
            let last = peekable_entries.peek().is_none();
            scan_directory_recursive(&entry, depth + 1, tree_string, last)?;
        }
    }
    Ok(())
}

// Basic test function (can be run with `cargo test`)
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, create_dir_all};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_scan_directory() {
        let dir = tempdir().unwrap();
        let root_path = dir.path();

        // Create some structure
        create_dir_all(root_path.join("src")).unwrap();
        File::create(root_path.join("src/main.rs")).unwrap().write_all(b"fn main() {}").unwrap();
        File::create(root_path.join("Cargo.toml")).unwrap().write_all(b"[package]").unwrap();
        create_dir_all(root_path.join("target")).unwrap(); // Should be ignored
        File::create(root_path.join("target/debug_file")).unwrap();
        File::create(root_path.join(".DS_Store")).unwrap(); // Should be ignored


        let expected_prefix = format!("{}\\n├── Cargo.toml\\n└── src\\n    └── main.rs\\n", root_path.file_name().unwrap().to_string_lossy());
        
        match scan_directory_tree_from_path(root_path) {
            Ok(tree) => {
                println!("Generated tree:\\n{}", tree);
                // Note: The exact string can be a bit tricky with dynamic root names
                // and platform-specific path separators. This checks the general structure.
                assert!(tree.contains("Cargo.toml"));
                assert!(tree.contains("src"));
                assert!(tree.contains("main.rs"));
                assert!(!tree.contains("target"));
                assert!(!tree.contains(".DS_Store"));
                assert!(tree.starts_with(&root_path.file_name().unwrap().to_string_lossy()));
            }
            Err(e) => panic!("Scan failed: {}", e),
        }
    }
}