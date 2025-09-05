//! # mdfmt
//!
//! A fast, lightweight Markdown formatter that intelligently cleans up your markdown files
//! while preserving structure and content.
//!
//! ## Features
//!
//! - Removes multiple consecutive blank lines while preserving document structure
//! - Ensures proper spacing around headings, code blocks, and list markers
//! - Protects frontmatter and code blocks from formatting changes
//! - Handles empty file detection and removal
//! - Processes files recursively with parallel execution
//!
//! ## Example
//!
//! ```rust
//! use mdfmt::process_md::remove_multiple_blank_lines;
//!
//! let input = "Line 1\n\n\n\nLine 2\n\nLine 3";
//! let output = remove_multiple_blank_lines(input);
//! assert_eq!(output, "Line 1\n\nLine 2\n\nLine 3");
//! ```
//!
//! ## Modules
//!
//! - [`find_md_files`] - Functions for finding Markdown files in directories
//! - [`process_md`] - Core formatting and processing functions

pub mod find_md_files;
pub mod process_md;

pub use find_md_files::find_md_files;
pub use process_md::{process_md_file, remove_multiple_blank_lines};
