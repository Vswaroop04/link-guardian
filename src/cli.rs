// src/cli.rs
// =============================================================================
// This file defines our command-line interface using the `clap` crate.
//
// clap is a popular Rust library for parsing command-line arguments.
// We use the "derive" API which lets us define the CLI structure using
// Rust structs and attributes (the #[...] things).
//
// Rust concepts:
// - Structs: Custom data types that group related data
// - Enums: Types that can be one of several variants
// - Derive macros: Automatically generate code for our types
// =============================================================================

use clap::{Parser, Subcommand};

// This struct represents our entire CLI application
//
// #[derive(Parser)] tells clap to automatically generate parsing code
// The #[command(...)] attributes configure how the CLI behaves
#[derive(Parser, Debug)]
#[command(
    name = "link-guardian",
    version = "0.1.0",
    about = "A CLI tool to scan for broken links in GitHub repos and websites",
    long_about = "link-guardian scans GitHub repositories and websites to find broken or redirected links. \
                  It's perfect for CI/CD pipelines to ensure your documentation stays up-to-date."
)]
pub struct Cli {
    // The #[command(subcommand)] attribute tells clap that this field
    // will hold one of the subcommands defined in the Commands enum
    #[command(subcommand)]
    pub command: Commands,
}

// This enum defines our subcommands (github, site)
//
// Each variant represents a different subcommand the user can run
// The fields inside each variant become the arguments for that subcommand
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan a GitHub repository for broken links in README and docs/
    ///
    /// Example: link-guardian github https://github.com/rust-lang/rust
    Github {
        /// GitHub repository URL (e.g., https://github.com/user/repo)
        ///
        /// This is a positional argument (required, no flag needed)
        repo_url: String,

        /// Output results in JSON format instead of a table
        ///
        /// This is an optional flag: --json
        /// #[arg(long)] creates a flag from the field name
        #[arg(long)]
        json: bool,
    },

    /// Scan a website for broken links
    ///
    /// Example: link-guardian site https://example.com --max-depth 2
    Site {
        /// Website URL to scan (e.g., https://example.com)
        ///
        /// This is a positional argument (required)
        website_url: String,

        /// Output results in JSON format instead of a table
        ///
        /// This is an optional flag: --json
        #[arg(long)]
        json: bool,

        /// Maximum crawl depth (default: 1)
        ///
        /// This controls how many levels deep we crawl from the starting page
        /// Depth 1 = just the starting page
        /// Depth 2 = starting page + all pages it links to
        /// etc.
        ///
        /// #[arg(long, default_value_t = 1)] creates --max-depth flag with default value
        #[arg(long, default_value_t = 1)]
        max_depth: usize,
    },
}

// -----------------------------------------------------------------------------
// BEGINNER NOTES:
//
// 1. Why use structs and enums?
//    - Structs group related data (like the CLI arguments)
//    - Enums represent choices (like "github OR site")
//    - Both are core Rust types for organizing data
//
// 2. What are derive macros?
//    - #[derive(...)] automatically generates code for common operations
//    - Parser: generates CLI parsing logic
//    - Debug: generates code to print the struct for debugging
//
// 3. What does 'pub' mean?
//    - pub = public, meaning other modules can use this
//    - Without pub, items are private to this module
//
// 4. Why String instead of &str?
//    - String is owned (the struct owns the data)
//    - &str is borrowed (references data owned elsewhere)
//    - We use String here because we need to own the CLI arguments
//
// 5. What is usize?
//    - An unsigned integer type that's the size of a pointer
//    - Used for sizes, lengths, and indices
//    - On 64-bit systems, usize is 64 bits
// -----------------------------------------------------------------------------
