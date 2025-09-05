use glob::glob;
use std::path::{Path, PathBuf};

/// Find all markdown files recursively in the given directory.
///
/// # Arguments
///
/// * `search_dir` - The directory to search for markdown files
///
/// # Returns
///
/// A vector of `PathBuf` containing all found markdown files.
///
/// # Panics
///
/// Panics if the glob pattern is invalid (which should never happen with our static pattern).
pub fn find_md_files(search_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let pattern = format!("{}/**/*.md", search_dir.display());

    match glob(&pattern) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(path) => {
                        // Only include regular files, skip directories that might end with .md
                        if path.is_file() {
                            files.push(path);
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Error reading path in glob: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: Failed to create glob pattern '{}': {}", pattern, e);
        }
    }

    // Sort files for consistent output
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;

    #[test]
    fn test_find_md_files_empty_directory() {
        // Use a temporary directory in the system temp location
        let temp_dir = env::temp_dir().join("mdfmt_test_empty");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).ok();
        }
        fs::create_dir_all(&temp_dir).unwrap();

        let files = find_md_files(&temp_dir);
        assert!(files.is_empty());

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_find_md_files_with_files() {
        let temp_dir = env::temp_dir().join("mdfmt_test_with_files");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).ok();
        }

        // Create test files
        let subdir = temp_dir.join("subdir");
        fs::create_dir_all(&subdir).unwrap();

        let md_file = temp_dir.join("test.md");
        let txt_file = temp_dir.join("test.txt");

        fs::write(&md_file, "# Test").unwrap();
        fs::write(&txt_file, "Not markdown").unwrap();
        fs::write(subdir.join("nested.md"), "# Nested").unwrap();

        let files = find_md_files(&temp_dir);
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.file_name().unwrap() == "test.md"));
        assert!(files.iter().any(|p| p.file_name().unwrap() == "nested.md"));

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }
}
