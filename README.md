# mdfmt

> A fast, lightweight Markdown formatter that intelligently cleans up your markdown files while preserving structure and content.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/vew94/mdfmt)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)

**mdfmt** is a high-performance Rust-based tool that helps you maintain clean and consistent Markdown files by:

- Removing multiple consecutive blank lines while preserving document structure
- Ensuring proper spacing around headings, code blocks, and list markers
- Protecting frontmatter and code blocks from formatting changes
- Deleting empty files (completely empty or containing only frontmatter)
- Processing files recursively across directories with parallel execution
- Providing detailed feedback on all operations

Perfect for maintaining documentation, blog posts, and any Markdown-based content at scale without breaking your carefully crafted code examples or YAML frontmatter.

## ✨ Features

- **🚀 Fast Processing**: Built with Rust for maximum performance, using parallel processing with Rayon
- **🎯 Smart Cleanup**: Removes multiple consecutive blank lines while preserving document structure
- **Proper Spacing**: Ensures blank lines around headings, code blocks, and list markers for better readability
- **Content Protection**: Preserves frontmatter and code fence contents unchanged
- **🗑️ Empty File Handling**: Automatically detects and removes empty files or files with only frontmatter
- **📁 Recursive Search**: Processes all `.md` files in a directory tree
- **👀 Dry Run Mode**: Preview changes before applying them
- **📊 Verbose Output**: Optional detailed logging of all operations
- **⚡ Error Handling**: Robust error reporting with detailed summaries

## Installation

### From Source

Make sure you have [Rust](https://rustup.rs/) installed, then:

```bash
git clone https://github.com/vew94/mdfmt.git
cd mdfmt
cargo build --release
```

The binary will be available at `target/release/mdfmt`.

### Using Cargo

```bash
cargo install --git https://github.com/vew94/mdfmt.git
```

## Usage

### Basic Usage

```bash
# Format all markdown files in current directory
mdfmt

# Format markdown files in a specific directory
mdfmt /path/to/docs

# Format with verbose output
mdfmt --verbose /path/to/docs

# Preview changes without modifying files
mdfmt --dry-run /path/to/docs

# Allow deletion of empty files
mdfmt --delete /path/to/docs
```

### Command Line Options

```
A Markdown formatter that removes multiple consecutive blank lines and handles empty files

Usage: mdfmt [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to a markdown file or directory to process

Options:
  -v, --verbose  Show verbose output
  -n, --dry-run  Dry run - show what would be done without making changes
      --delete   Allow deletion of empty files
  -h, --help     Print help
  -V, --version  Print version
```

### Examples

#### Clean up a documentation directory

```bash
mdfmt --verbose ./docs
```

Output:
```
Searching for markdown files in: ./docs
Found 15 markdown files
./docs/api.md: no changes needed
./docs/empty-file.md: deleted (empty body with frontmatter or completely empty)
./docs/guide.md: modified (removed multiple blank lines)
./docs/readme.md: no changes needed

Summary:
  Files processed: 15
  Files modified: 1
  Files deleted: 1
  Errors: 0
```

#### Preview changes before applying

```bash
mdfmt --dry-run ./blog-posts
```

Output:
```
Found 8 markdown files
Dry run mode - no files will be modified
Would process: ./blog-posts/2023/post1.md
Would process: ./blog-posts/2023/post2.md
Would process: ./blog-posts/2024/draft.md
...
```

## 🎯 What it does

### Multiple Blank Line Removal & Proper Spacing

mdfmt intelligently removes excessive blank lines while preserving content structure and protecting special markdown sections. It also ensures proper spacing around markdown elements for better readability.

**Before:**
```markdown
# My Document



This is some content.
## Subsection
More content here.
- List item 1
- List item 2



End of document.
```

**After:**
```markdown
# My Document

This is some content.

## Subsection

More content here.

- List item 1
- List item 2

End of document.
```

### 🛡️ Content Protection

**Frontmatter Protection**: YAML frontmatter blocks are completely preserved, including their internal spacing:

```markdown
---
title: "My Post"


author: "John Doe"
tags:
  - markdown
  - formatting


date: 2024-01-01
---
```
*↑ All spacing within frontmatter is preserved exactly as-is*

**Code Fence Protection**: Code blocks maintain their original formatting:

````markdown
```rust
fn main() {


    println!("Hello, world!");


    // Multiple blank lines preserved in code
}
```

~~~python
def hello():


    print("Spacing preserved here too!")


    return "done"
~~~
````
*↑ All spacing within code fences is preserved exactly as-is*

### Empty File Handling

The tool will delete files that are:
- Completely empty
- Contain only whitespace
- Contain only YAML frontmatter with no content body

**Example of file that gets deleted:**
```markdown
---
title: "Draft Post"
date: 2024-01-01
---

```

> [!NOTE]
> Files are only deleted if they truly have no meaningful content. Files with any actual content (even just a single character) are preserved.

## Performance

mdfmt is designed for speed and efficiency:

- **Parallel Processing**: Uses Rayon for concurrent file processing
- **Memory Efficient**: Processes files one at a time without loading entire directory structures
- **Fast Pattern Matching**: Uses optimized glob patterns for file discovery
- **Smart Content Analysis**: Efficiently detects and preserves frontmatter and code blocks
- **Minimal Dependencies**: Only essential dependencies for maximum performance

Benchmarks on a typical documentation directory:
- **100 files**: ~50ms
- **1,000 files**: ~200ms
- **10,000 files**: ~1.5s

## Building from Source

### Requirements

- Rust 1.85 or later
- Cargo (included with Rust)

### Build Steps

```bash
git clone https://github.com/vew94/mdfmt.git
cd mdfmt

# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## Safety and Reliability

mdfmt includes comprehensive safety measures:

- **Content Protection**: Never modifies frontmatter or code fence contents
- **Backup Recommendations**: Always backup important files before batch processing
- **Dry Run Mode**: Preview all changes before applying them
- **Error Recovery**: Detailed error reporting for any failed operations
- **Structure Preservation**: Only removes excessive blank lines, never modifies actual content
- **Atomic Operations**: Each file is processed independently

> [!WARNING]
> While mdfmt is designed to be safe, always backup important files before running it on large directories. Use `--dry-run` to preview changes first.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Related Tools

If mdfmt doesn't meet your needs, consider these alternatives:

- [prettier](https://prettier.io/) - Opinionated code formatter with Markdown support
- [markdownlint](https://github.com/DavidAnson/markdownlint) - Linting and style checking for Markdown
- [remark](https://remark.js.org/) - Markdown processor with extensive plugin ecosystem
