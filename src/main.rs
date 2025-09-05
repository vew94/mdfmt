use clap::Parser;
use rayon::prelude::*;
use std::path::Path;
use std::process;

mod find_md_files;
mod process_md;

/// A Markdown formatter that removes multiple consecutive blank lines and handles empty files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to a markdown file or directory to process
    #[arg(value_name = "PATH")]
    path: Option<String>,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dry run - show what would be done without making changes
    #[arg(short = 'n', long)]
    dry_run: bool,
}

fn main() {
    let cli = Args::parse();

    let search_dir = if let Some(p) = cli.path {
        let input_path = Path::new(&p);
        if input_path.is_file() {
            input_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| match std::env::current_dir() {
                    Ok(dir) => dir,
                    Err(e) => {
                        eprintln!("Error: Failed to get current directory: {}", e);
                        process::exit(1);
                    }
                })
        } else if input_path.is_dir() {
            input_path.to_path_buf()
        } else {
            eprintln!("Error: Path '{}' does not exist or is not accessible", p);
            process::exit(1);
        }
    } else {
        match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Error: Failed to get current directory: {}", e);
                process::exit(1);
            }
        }
    };

    if cli.verbose {
        println!("Searching for markdown files in: {}", search_dir.display());
    }

    let md_files = find_md_files::find_md_files(&search_dir);

    if md_files.is_empty() {
        println!("No markdown files found in {}", search_dir.display());
        return;
    }

    println!(
        "Found {} markdown file{}",
        md_files.len(),
        if md_files.len() == 1 { "" } else { "s" }
    );

    if cli.dry_run {
        println!("Dry run mode - no files will be modified");
        for file in &md_files {
            println!("Would process: {}", file.display());
        }
        return;
    }

    let results: Vec<_> = md_files
        .par_iter()
        .map(|path| {
            let result = match process_md::process_md_file(path) {
                Ok((deleted, modified)) => {
                    if deleted {
                        Ok("deleted (empty body with frontmatter or completely empty)".to_string())
                    } else if modified {
                        Ok("modified (removed multiple blank lines)".to_string())
                    } else {
                        Ok("no changes needed".to_string())
                    }
                }
                Err(e) => Err(format!("error: {}", e)),
            };
            (path, result)
        })
        .collect();

    let mut deleted_count = 0;
    let mut modified_count = 0;
    let mut error_count = 0;

    for (path, result) in results {
        match result {
            Ok(status) => {
                if cli.verbose || !status.contains("no changes needed") {
                    println!("{}: {}", path.display(), status);
                }
                if status.contains("deleted") {
                    deleted_count += 1;
                } else if status.contains("modified") {
                    modified_count += 1;
                }
            }
            Err(error) => {
                eprintln!("{}: {}", path.display(), error);
                error_count += 1;
            }
        }
    }

    // Print summary
    println!();
    println!("Summary:");
    println!("  Files processed: {}", md_files.len());
    println!("  Files modified: {}", modified_count);
    println!("  Files deleted: {}", deleted_count);
    println!("  Errors: {}", error_count);

    if error_count > 0 {
        process::exit(1);
    }
}
