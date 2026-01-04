// src/main.rs
// =============================================================================
// This is the entry point of our CLI application.
//
// What happens here:
// 1. Parse command-line arguments using clap
// 2. Dispatch to the appropriate subcommand handler
// 3. Collect results and print them
// 4. Exit with proper code (0 = success, 1 = broken links, 2 = error)
//
// Rust concepts used:
// - async/await: Because we need to make many network requests concurrently
// - Result<T, E>: For error handling (T = success type, E = error type)
// - match: Pattern matching to handle different subcommands
// =============================================================================

// Module declarations - tells Rust about our other source files
mod cli;           // src/cli.rs - command-line parsing
mod checker;       // src/checker/ - link checking logic
mod github;        // src/github/ - GitHub-specific functionality
mod crawl;         // src/crawl/ - website crawling logic

// Import items we need from our modules
use cli::{Cli, Commands};
use clap::Parser;  // Parser trait enables the parse() method

// anyhow::Result is like std::result::Result but simpler for applications
// It lets us return any error type with the ? operator
use anyhow::Result;

// The #[tokio::main] attribute transforms our async main into a real main function
// It creates a tokio runtime and runs our async code inside it
#[tokio::main]
async fn main() {
    // Run our application logic and capture the exit code
    // std::process::exit() terminates the program with the given code
    let exit_code = match run().await {
        Ok(code) => code,
        Err(e) => {
            // If an unexpected error occurred, print it and exit with code 2
            eprintln!("Error: {}", e);
            2
        }
    };

    std::process::exit(exit_code);
}

// This is the main application logic
// Returns:
//   Ok(0) = no broken links
//   Ok(1) = broken links found
//   Ok(2) = internal error
//   Err = unexpected error
async fn run() -> Result<i32> {
    // Parse command-line arguments into our Cli struct
    // This will automatically handle --help, --version, etc.
    let cli = Cli::parse();

    // Match on which subcommand was used
    // Each branch handles a different command (github, site)
    match cli.command {
        Commands::Github { repo_url, json } => {
            // Call our github scanning function
            handle_github_scan(&repo_url, json).await
        }
        Commands::Site { website_url, json, max_depth } => {
            // Call our website scanning function
            handle_site_scan(&website_url, json, max_depth).await
        }
    }
}

// Handles the 'github' subcommand
// Parameters:
//   repo_url: GitHub repository URL (e.g., "https://github.com/user/repo")
//   json: whether to output JSON format
async fn handle_github_scan(repo_url: &str, json: bool) -> Result<i32> {
    println!("🔍 Scanning GitHub repository: {}", repo_url);

    // Fetch README and docs from the repository
    let files = github::fetch_repo_files(repo_url).await?;

    if files.is_empty() {
        println!("⚠️  No markdown files found in repository");
        return Ok(0);
    }

    println!("📄 Found {} file(s) to scan", files.len());

    // Extract all links from markdown files
    let mut all_links = Vec::new();
    for (filename, content) in &files {
        let links = checker::extract_markdown_links(content);
        println!("   {} links found in {}", links.len(), filename);
        all_links.extend(links);
    }

    if all_links.is_empty() {
        println!("✅ No links found to check");
        return Ok(0);
    }

    println!("\n🌐 Checking {} unique link(s)...\n", all_links.len());

    // Check all links for broken status
    let results = checker::check_links(all_links).await;

    // Print results and determine exit code
    print_results(&results, json)?;

    // Count how many links are broken
    let broken_count = results.iter()
        .filter(|r| !r.is_ok())
        .count();

    if broken_count > 0 {
        Ok(1)  // Exit code 1 = broken links found
    } else {
        Ok(0)  // Exit code 0 = all good
    }
}

// Handles the 'site' subcommand
// Parameters:
//   website_url: Website URL to crawl (e.g., "https://example.com")
//   json: whether to output JSON format
//   max_depth: how many levels deep to crawl (default: 1)
async fn handle_site_scan(website_url: &str, json: bool, max_depth: usize) -> Result<i32> {
    println!("🔍 Scanning website: {}", website_url);
    println!("📊 Max crawl depth: {}", max_depth);

    // Crawl the website and collect all pages
    let pages = crawl::crawl_website(website_url, max_depth).await?;

    println!("📄 Crawled {} page(s)", pages.len());

    // Extract all links from all pages
    let mut all_links = Vec::new();
    for (page_url, html) in &pages {
        let links = checker::extract_html_links(html, page_url);
        println!("   {} links found on {}", links.len(), page_url);
        all_links.extend(links);
    }

    if all_links.is_empty() {
        println!("✅ No links found to check");
        return Ok(0);
    }

    // Remove duplicates by converting to a HashSet and back
    let unique_links: std::collections::HashSet<_> = all_links.into_iter().collect();
    let all_links: Vec<_> = unique_links.into_iter().collect();

    println!("\n🌐 Checking {} unique link(s)...\n", all_links.len());

    // Check all links for broken status
    let results = checker::check_links(all_links).await;

    // Print results and determine exit code
    print_results(&results, json)?;

    // Count broken links
    let broken_count = results.iter()
        .filter(|r| !r.is_ok())
        .count();

    if broken_count > 0 {
        Ok(1)  // Exit code 1 = broken links found
    } else {
        Ok(0)  // Exit code 0 = all good
    }
}

// Prints the results either as a table or JSON
// Parameters:
//   results: slice of LinkCheckResult structs
//   json: whether to output JSON format
fn print_results(results: &[checker::LinkCheckResult], json: bool) -> Result<()> {
    if json {
        // Serialize results to JSON and print
        let json_output = serde_json::to_string_pretty(results)?;
        println!("{}", json_output);
    } else {
        // Print human-readable table
        print_table(results);
    }
    Ok(())
}

// Prints results as a human-readable table in the terminal
fn print_table(results: &[checker::LinkCheckResult]) {
    // Print table header
    println!("{:<60} {:<15} {:<30}", "URL", "STATUS", "MESSAGE");
    println!("{}", "=".repeat(105));

    // Print each result
    for result in results {
        let status_display = format_status(&result.status);
        let message = result.message.as_deref().unwrap_or("");

        // Truncate URL if too long for display
        let url_display = if result.url.len() > 57 {
            format!("{}...", &result.url[..57])
        } else {
            result.url.clone()
        };

        println!("{:<60} {:<15} {:<30}", url_display, status_display, message);
    }

    println!();

    // Print summary
    let ok_count = results.iter().filter(|r| r.is_ok()).count();
    let broken_count = results.len() - ok_count;

    println!("📊 Summary:");
    println!("   ✅ OK: {}", ok_count);
    println!("   ❌ Broken: {}", broken_count);
    println!("   📋 Total: {}", results.len());
}

// Formats the status enum as a colored string
// (We'll add actual colors in future iterations)
fn format_status(status: &checker::LinkStatus) -> String {
    match status {
        checker::LinkStatus::Ok => "✅ OK".to_string(),
        checker::LinkStatus::Redirect(_) => "🔀 REDIRECT".to_string(),
        checker::LinkStatus::Broken => "❌ BROKEN".to_string(),
        checker::LinkStatus::Timeout => "⏱️  TIMEOUT".to_string(),
        checker::LinkStatus::SslError => "🔒 SSL ERROR".to_string(),
        checker::LinkStatus::TooManyRedirects => "🔁 TOO MANY REDIRECTS".to_string(),
        checker::LinkStatus::DnsError => "🌐 DNS ERROR".to_string(),
        checker::LinkStatus::Error => "⚠️  ERROR".to_string(),
    }
}
