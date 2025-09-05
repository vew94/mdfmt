//! Markdown processing functionality for removing multiple blank lines and handling empty files.
//!
//! This module provides functions to process markdown files by removing excessive blank lines
//! while preserving important formatting like frontmatter and code blocks.

use std::fs;
use std::io;
use std::path::Path;

/// Process a markdown file to remove multiple consecutive blank lines and handle empty files.
///
/// This function reads a markdown file, processes its content to remove excessive blank lines,
/// and optionally deletes empty files. It preserves frontmatter content and code fence blocks.
///
/// Returns a tuple of (deleted, modified) where:
/// - `deleted` indicates if the file was deleted (empty body with only frontmatter)
/// - `modified` indicates if the file was modified (removed multiple blank lines)
///
/// # Arguments
///
/// * `path` - Path to the markdown file to process
/// * `allow_delete` - Whether to allow deletion of empty files
///
/// # Examples
///
/// ```rust,no_run
/// use mdfmt::process_md::process_md_file;
/// use std::path::Path;
///
/// // Process a file without allowing deletion
/// let (deleted, modified) = process_md_file(Path::new("example.md"), false)?;
/// if modified {
///     println!("File was modified");
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Returns an `io::Error` if the file cannot be read or written.
pub fn process_md_file<P: AsRef<Path>>(path: P, allow_delete: bool) -> io::Result<(bool, bool)> {
    let path = path.as_ref();
    let original_content = fs::read_to_string(path)?;

    if original_content.trim().is_empty() {
        // Delete completely empty files only if deletion is allowed
        if allow_delete {
            fs::remove_file(path)?;
            return Ok((true, false));
        } else {
            // Skip processing but don't delete
            return Ok((false, false));
        }
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

    // If body is empty or only whitespace and we have frontmatter, delete the file if allowed
    if frontmatter.is_some() && body.trim().is_empty() {
        if allow_delete {
            fs::remove_file(path)?;
            return Ok((true, false));
        } else {
            // Skip processing but don't delete
            return Ok((false, false));
        }
    }
    // Process content to remove multiple consecutive blank lines
    let processed_content = remove_multiple_blank_lines(&original_content);

    // Check if content was modified
    if processed_content != original_content {
        fs::write(path, processed_content)?;
        Ok((false, true))
    } else {
        Ok((false, false))
    }
}

/// Remove multiple consecutive blank lines and ensure proper spacing around markdown elements.
/// This function preserves frontmatter and code fence contents while adding blank lines
/// around headings, code fences, and list markers.
///
/// # Arguments
///
/// * `content` - The content to process
///
/// # Returns
///
/// The processed content with multiple blank lines reduced to single blank lines
/// and proper spacing around markdown elements.
///
/// # Examples
///
/// Basic usage:
/// ```
/// use mdfmt::process_md::remove_multiple_blank_lines;
///
/// let input = "Line 1\n\n\n\nLine 2\n\nLine 3";
/// let output = remove_multiple_blank_lines(input);
/// assert_eq!(output, "Line 1\n\nLine 2\n\nLine 3");
/// ```
///
/// Preserving frontmatter:
/// ```
/// use mdfmt::process_md::remove_multiple_blank_lines;
///
/// let input = "---\ntitle: Test\n\n\nauthor: Me\n---\n\n\n\nContent";
/// let output = remove_multiple_blank_lines(input);
/// assert_eq!(output, "---\ntitle: Test\n\n\nauthor: Me\n---\n\nContent");
/// ```
///
/// Adding spacing around headings:
/// ```
/// use mdfmt::process_md::remove_multiple_blank_lines;
///
/// let input = "Text\n# Heading\nMore text";
/// let output = remove_multiple_blank_lines(input);
/// assert_eq!(output, "Text\n\n# Heading\n\nMore text");
/// ```
pub fn remove_multiple_blank_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut prev_was_empty = false;
    let mut in_frontmatter = false;
    let mut in_code_fence = false;
    let mut code_fence_marker = "";

    // Helper functions for detecting markdown elements
    let is_heading = |line: &str| {
        let trimmed = line.trim();
        trimmed.starts_with('#') && trimmed.chars().take_while(|c| *c == '#').count() <= 6
    };
    let is_list_marker = |line: &str| {
        let trimmed = line.trim();
        trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
            || (trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
                && trimmed.contains(". "))
    };

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
            if lines.get(i + 1).is_some_and(|next| !next.trim().is_empty()) {
                result.push("");
            }
            continue;
        }

        // Check for code fence start/end
        if !in_frontmatter {
            let trimmed = line.trim();
            if (trimmed.starts_with("```") || trimmed.starts_with("~~~")) && !in_code_fence {
                // Insert blank line before code fence if previous line is not blank
                if !result.is_empty() && result.last().is_some_and(|l| !l.trim().is_empty()) {
                    result.push("");
                }
                // Starting a code fence
                in_code_fence = true;
                code_fence_marker = if trimmed.starts_with("```") {
                    "```"
                } else {
                    "~~~"
                };
                result.push(*line);
                prev_was_empty = false;
                continue;
            } else if in_code_fence
                && (trimmed.starts_with(code_fence_marker)
                    && trimmed.len() >= code_fence_marker.len())
            {
                // Ending a code fence - must start with the same marker
                in_code_fence = false;
                code_fence_marker = "";
                result.push(*line);
                // Insert blank line after code fence if next line is not blank
                if lines.get(i + 1).is_some_and(|next| !next.trim().is_empty()) {
                    result.push("");
                }
                prev_was_empty = false;
                continue;
            }
        }

        // If we're inside frontmatter or code fence, don't process blank lines
        if in_frontmatter || in_code_fence {
            // Special handling for code fence: remove blank lines immediately after opening or before closing
            if in_code_fence {
                let is_blank = line.trim().is_empty();

                // Check if this is immediately after code fence start
                let prev_line = if i > 0 { lines.get(i - 1) } else { None };
                let prev_was_fence_start = prev_line
                    .map(|l| {
                        let trimmed = l.trim();
                        (trimmed.starts_with("```") || trimmed.starts_with("~~~"))
                            && !in_frontmatter
                    })
                    .unwrap_or(false);

                // Check if next line is code fence end
                let next_line = lines.get(i + 1);
                let next_is_fence_end = next_line
                    .map(|l| {
                        let trimmed = l.trim();
                        trimmed.starts_with(code_fence_marker)
                            && trimmed.len() >= code_fence_marker.len()
                    })
                    .unwrap_or(false);

                // Skip blank line if it's immediately after fence start or before fence end
                if is_blank && (prev_was_fence_start || next_is_fence_end) {
                    continue;
                }
            }

            result.push(*line);
            prev_was_empty = false;
            continue;
        }

        // Insert blank line before heading or list group start if previous line is not blank
        let is_list_group_start = is_list_marker(line)
            && (i == 0 || !is_list_marker(lines.get(i.saturating_sub(1)).unwrap_or(&"")));

        if (is_heading(line) || is_list_group_start)
            && !result.is_empty()
            && result.last().is_some_and(|l| !l.trim().is_empty())
        {
            result.push("");
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

        // Insert blank line after heading or list group end if next line is not blank
        let is_list_group_end = is_list_marker(line)
            && !lines
                .get(i + 1)
                .map(|next| is_list_marker(next))
                .unwrap_or(false);

        if (is_heading(line) || is_list_group_end)
            && lines.get(i + 1).is_some_and(|next| !next.trim().is_empty())
        {
            result.push("");
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
        let input =
            "---\ntitle: Test\n\n\n\nauthor: Me\n---\n\n\n\nContent here\n\n\n\nMore content";
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
        let expected =
            "Some text\n\n~~~python\ndef hello():\n\n\n\n    print(\"Hello\")\n~~~\n\nMore text";
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
        // 5. "\n\n\n\ntext" is inside the new code fence - first blank line after fence start is removed
        let expected = "```\ncode\n```inner\n\nmore code\n\n```\n\n\ntext";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_code_fence_no_blank_after_start_before_end() {
        let input = "Text\n```\n\ncode line\n\n```\nText";
        let expected = "Text\n\n```\ncode line\n```\n\nText";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_code_fence_preserve_internal_blanks() {
        let input = "Text\n```\nline1\n\n\nline2\n```\nText";
        let expected = "Text\n\n```\nline1\n\n\nline2\n```\n\nText";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_heading_and_list_blank_lines() {
        let input = "Text\n# Heading1\nText\n- item1\n- item2\nText\n1. item3\n2. item4\nText\n* item5\n* item6\nText\n+ item7\n+ item8\nText";
        let expected = "Text\n\n# Heading1\n\nText\n\n- item1\n- item2\n\nText\n\n1. item3\n2. item4\n\nText\n\n* item5\n* item6\n\nText\n\n+ item7\n+ item8\n\nText";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_code_fence_blank_lines() {
        let input = "Text\n```rust\nfn main() {}\n```\nText";
        let expected = "Text\n\n```rust\nfn main() {}\n```\n\nText";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_no_extra_blank_lines_for_existing() {
        let input = "Text\n\n# Heading\n\nText\n\n- item\n\nText";
        let expected = "Text\n\n# Heading\n\nText\n\n- item\n\nText";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_consecutive_list_items() {
        let input = "Text\n- A\n- B\n- C\nText";
        let expected = "Text\n\n- A\n- B\n- C\n\nText";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }

    #[test]
    fn test_mixed_list_types() {
        let input = "Text\n- A\n* B\n+ C\n1. D\n2. E\nText";
        let expected = "Text\n\n- A\n* B\n+ C\n1. D\n2. E\n\nText";
        assert_eq!(remove_multiple_blank_lines(input), expected);
    }
}
