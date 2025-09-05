use std::fs;
use std::io;
use std::path::Path;

/// Process a markdown file to remove multiple consecutive blank lines and handle empty files.
///
/// Returns a tuple of (deleted, modified) where:
/// - `deleted` indicates if the file was deleted (empty body with only frontmatter)
/// - `modified` indicates if the file was modified (removed multiple blank lines)
///
/// # Arguments
///
/// * `path` - Path to the markdown file to process
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be read or written.
pub fn process_md_file<P: AsRef<Path>>(path: P) -> io::Result<(bool, bool)> {
    let path = path.as_ref();
    let original_content = fs::read_to_string(path)?;

    if original_content.trim().is_empty() {
        // Delete completely empty files
        fs::remove_file(path)?;
        return Ok((true, false));
    }

    // Check if file has frontmatter
    let (frontmatter, body) = if let Some(stripped) = original_content.strip_prefix("---\n") {
        if let Some(end_pos) = stripped.find("\n---\n") {
            let frontmatter_end = end_pos + 8; // 4 for initial "---\n" + 4 for "\n---\n"
            let frontmatter = &original_content[..frontmatter_end];
            let body = &original_content[frontmatter_end..];
            (Some(frontmatter), body)
        } else {
            (None, original_content.as_str())
        }
    } else {
        (None, original_content.as_str())
    };

    // If body is empty or only whitespace and we have frontmatter, delete the file
    if frontmatter.is_some() && body.trim().is_empty() {
        fs::remove_file(path)?;
        return Ok((true, false));
    }    // Process content to remove multiple consecutive blank lines
    let processed_content = remove_multiple_blank_lines(&original_content);

    // Check if content was modified
    if processed_content != original_content {
        fs::write(path, processed_content)?;
        Ok((false, true))
    } else {
        Ok((false, false))
    }
}

/// Remove multiple consecutive blank lines, keeping only single blank lines.
///
/// # Arguments
///
/// * `content` - The content to process
///
/// # Returns
///
/// The processed content with multiple blank lines reduced to single blank lines.
fn remove_multiple_blank_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut prev_was_empty = false;

    for line in lines {
        let is_empty = line.trim().is_empty();

        if is_empty {
            if !prev_was_empty {
                result.push(line);
            }
            prev_was_empty = true;
        } else {
            result.push(line);
            prev_was_empty = false;
        }
    }

    // Preserve the original ending (newline or not)
    let result_content = result.join("\n");
    if content.ends_with('\n') && !result_content.ends_with('\n') {
        format!("{}\n", result_content)
    } else if !content.ends_with('\n') && result_content.ends_with('\n') {
        result_content.trim_end_matches('\n').to_string()
    } else {
        result_content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_multiple_blank_lines() {
        let input = "Line 1\n\n\n\nLine 2\n\nLine 3\n\n\n";
        let expected = "Line 1\n\nLine 2\n\nLine 3\n";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_remove_multiple_blank_lines_no_change() {
        let input = "Line 1\nLine 2\n\nLine 3";
        assert_eq!(remove_multiple_blank_lines(input), input);
    }

    #[test]
    fn test_remove_multiple_blank_lines_preserve_ending() {
        let input_no_newline = "Line 1\n\n\nLine 2";
        let expected_no_newline = "Line 1\n\nLine 2";
        assert_eq!(
            remove_multiple_blank_lines(input_no_newline),
            expected_no_newline
        );
    }
}
