// src/github/fetch.rs
// =============================================================================
// This module fetches markdown files from GitHub repositories.
//
// Strategy:
// - Parse the GitHub URL to extract owner and repo name
// - Fetch README.md from the repo root
// - Use raw.githubusercontent.com which serves raw file contents
//
// Why not the GitHub API?
// - The API requires authentication for higher rate limits
// - For MVP, raw file access is simpler
// - For production, you'd want to use the API (see stretch goals)
//
// Rust concepts:
// - async functions: For network I/O
// - Result: For error handling
// - Vec and HashMap: For storing data
// - String parsing: To extract owner/repo from URL
// =============================================================================

use anyhow::{anyhow, Result};
use reqwest::Client;

// Fetches markdown files from a GitHub repository
//
// Parameters:
//   repo_url: GitHub repository URL (e.g., "https://github.com/rust-lang/rust")
//
// Returns: Result<Vec<(String, String)>>
//   Success: Vec of (filename, content) tuples
//   Error: If URL is invalid or fetching fails
//
// Currently fetches:
//   - README.md from repo root
//   - (Future: files from docs/ directory)
pub async fn fetch_repo_files(repo_url: &str) -> Result<Vec<(String, String)>> {
    // Parse the URL to extract owner and repo name
    let (owner, repo) = parse_github_url(repo_url)?;

    // Create HTTP client for making requests
    let client = Client::new();

    let mut files = Vec::new();

    // Try to fetch README.md
    // Note: GitHub repos can have README.md, Readme.md, readme.md, etc.
    // For MVP, we'll try README.md (most common)
    let readme_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/README.md",
        owner, repo
    );

    match fetch_file(&client, &readme_url).await {
        Ok(content) => {
            files.push(("README.md".to_string(), content));
        }
        Err(_) => {
            // If main branch doesn't work, try master branch
            let readme_url = format!(
                "https://raw.githubusercontent.com/{}/{}/master/README.md",
                owner, repo
            );

            match fetch_file(&client, &readme_url).await {
                Ok(content) => {
                    files.push(("README.md".to_string(), content));
                }
                Err(e) => {
                    eprintln!("Warning: Could not fetch README.md: {}", e);
                }
            }
        }
    }

    // Future enhancement: Also fetch from docs/ directory
    // Would require using GitHub API to list directory contents

    Ok(files)
}

// Parses a GitHub URL to extract owner and repository name
//
// Supported formats:
//   - https://github.com/owner/repo
//   - https://github.com/owner/repo.git
//   - github.com/owner/repo
//
// Returns: (owner, repo) tuple
//
// Example:
//   "https://github.com/rust-lang/rust" -> ("rust-lang", "rust")
fn parse_github_url(url: &str) -> Result<(String, String)> {
    // Remove common prefixes
    let url = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    // Should start with github.com
    if !url.starts_with("github.com/") {
        return Err(anyhow!("Not a GitHub URL: {}", url));
    }

    // Remove "github.com/" prefix
    let path = url.trim_start_matches("github.com/");

    // Split by '/' to get owner and repo
    let parts: Vec<&str> = path.split('/').collect();

    if parts.len() < 2 {
        return Err(anyhow!("Invalid GitHub URL format: {}", url));
    }

    let owner = parts[0].to_string();
    let mut repo = parts[1].to_string();

    // Remove .git suffix if present
    if repo.ends_with(".git") {
        repo = repo.trim_end_matches(".git").to_string();
    }

    Ok((owner, repo))
}

// Fetches content from a URL
//
// Parameters:
//   client: reqwest HTTP client
//   url: URL to fetch
//
// Returns: String content or error
async fn fetch_file(client: &Client, url: &str) -> Result<String> {
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch {}: HTTP {}",
            url,
            response.status()
        ));
    }

    let content = response.text().await?;
    Ok(content)
}

// -----------------------------------------------------------------------------
// BEGINNER NOTES:
//
// 1. What is anyhow::Result?
//    - A type alias for Result<T, anyhow::Error>
//    - anyhow::Error can hold any error type
//    - Great for application code (vs libraries which should use specific errors)
//    - The ? operator works seamlessly with it
//
// 2. What is the ? operator?
//    - Shorthand for error propagation
//    - If Result is Ok(value), extracts value
//    - If Result is Err(e), returns early with the error
//    - Example: let x = function()?; is shorthand for:
//      let x = match function() {
//          Ok(v) => v,
//          Err(e) => return Err(e),
//      };
//
// 3. Why &str for parameters but String for return?
//    - &str = borrowed string slice, no allocation
//    - String = owned string, allocated on heap
//    - Take &str when you just need to read
//    - Return String when you create new data
//
// 4. What is format! macro?
//    - Creates a new String by formatting
//    - Like println! but returns a String instead of printing
//    - Uses {} placeholders for values
//
// 5. What is Vec<(String, String)>?
//    - Vec = growable array
//    - (String, String) = tuple with two strings
//    - Vec of tuples is a simple way to pair related data
//    - Each tuple is (filename, file_content)
//
// 6. Why .to_string()?
//    - Converts &str to String (borrowed to owned)
//    - Needed because we're returning owned data
//    - Creates a new allocation with the string data
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        let (owner, repo) = parse_github_url("https://github.com/rust-lang/rust").unwrap();
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_github_url_with_git() {
        let (owner, repo) = parse_github_url("https://github.com/user/repo.git").unwrap();
        assert_eq!(owner, "user");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = parse_github_url("https://gitlab.com/user/repo");
        assert!(result.is_err());
    }
}
