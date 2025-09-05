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
            // end_pos is relative to stripped content, so we need to add back the initial "---\n" (4 chars)
            // and then add the length of "\n---\n" (5 chars) to get the position after frontmatter
            let frontmatter_end = 4 + end_pos + 5; // "---\n" + content + "\n---\n"
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
/// This function preserves frontmatter and code fence contents.
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
    let mut in_frontmatter = false;
    let mut in_code_fence = false;
    let mut code_fence_marker = "";

    for (i, line) in lines.iter().enumerate() {
        // Check for frontmatter start/end
        if i == 0 && line.trim() == "---" {
            in_frontmatter = true;
            result.push(*line);
            continue;
        } else if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            result.push(*line);
            // Add a blank line after frontmatter ends only if next line is not already blank
            if lines.get(i + 1).map_or(false, |next| !next.trim().is_empty()) {
                result.push("");
            }
            continue;
        }

        // Check for code fence start/end
        if !in_frontmatter {
            let trimmed = line.trim();
            if (trimmed.starts_with("```") || trimmed.starts_with("~~~")) && !in_code_fence {
                // Starting a code fence
                in_code_fence = true;
                code_fence_marker = if trimmed.starts_with("```") { "```" } else { "~~~" };
                result.push(*line);
                prev_was_empty = false;
                continue;
            } else if in_code_fence && (trimmed.starts_with(code_fence_marker) && trimmed.len() >= code_fence_marker.len()) {
                // Ending a code fence - must start with the same marker
                in_code_fence = false;
                code_fence_marker = "";
                result.push(*line);
                prev_was_empty = false;
                continue;
            }
        }

        // If we're inside frontmatter or code fence, don't process blank lines
        if in_frontmatter || in_code_fence {
            result.push(*line);
            prev_was_empty = false;
            continue;
        }

        // Normal blank line processing for content outside protected areas
        let is_empty = line.trim().is_empty();

        if is_empty {
            if !prev_was_empty {
                result.push(*line);
            }
            prev_was_empty = true;
        } else {
            result.push(*line);
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

    #[test]
    fn test_preserve_frontmatter() {
        let input = "---\ntitle: Test\n\n\n\nauthor: Me\n---\n\n\n\nContent here\n\n\n\nMore content";
        let expected = "---\ntitle: Test\n\n\n\nauthor: Me\n---\n\nContent here\n\nMore content";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_preserve_code_fences() {
        let input = "Some text\n\n\n\n```rust\nfn main() {\n\n\n\n    println!(\"Hello\");\n}\n```\n\n\n\nMore text";
        let expected = "Some text\n\n```rust\nfn main() {\n\n\n\n    println!(\"Hello\");\n}\n```\n\nMore text";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_preserve_tilde_code_fences() {
        let input = "Some text\n\n\n\n~~~python\ndef hello():\n\n\n\n    print(\"Hello\")\n~~~\n\n\n\nMore text";
        let expected = "Some text\n\n~~~python\ndef hello():\n\n\n\n    print(\"Hello\")\n~~~\n\nMore text";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_frontmatter_and_code_fences_combined() {
        let input = "---\ntitle: Test\n\n\n\nauthor: Me\n---\n\n\n\nSome text\n\n\n\n```rust\nfn test() {\n\n\n\n    // code\n}\n```\n\n\n\nEnd";
        let expected = "---\ntitle: Test\n\n\n\nauthor: Me\n---\n\nSome text\n\n```rust\nfn test() {\n\n\n\n    // code\n}\n```\n\nEnd";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_nested_code_fences_not_supported() {
        // This test documents current behavior - nested code fences are not supported
        // The first closing fence will end the code block
        let input = "```\ncode\n```inner\nmore code\n```\n\n\n\ntext";
        // Breakdown:
        // 1. ``` starts code fence
        // 2. ```inner ends it (because it starts with ```)
        // 3. "more code" is normal text (no blank line processing needed)
        // 4. ``` starts a new code fence
        // 5. "\n\n\n\ntext" is inside the new code fence (preserved as-is)
        let expected = "```\ncode\n```inner\nmore code\n```\n\n\n\ntext";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }
}
