// src/github/mod.rs
// =============================================================================
// This module handles fetching files from GitHub repositories.
//
// Currently implements:
// - Parsing GitHub URLs to extract owner/repo
// - Fetching README.md and files from docs/ directory
// - Using raw.githubusercontent.com to get file contents
//
// Future enhancements (stretch goals):
// - Use GitHub API with octocrab for more robust access
// - Handle authentication for private repos
// - Support more file patterns
//
// Rust concepts:
// - Modules: Organizing related functionality
// - Public API: What other parts of the app can use
// =============================================================================

mod fetch;

// Re-export the main function from fetch.rs
pub use fetch::fetch_repo_files;
