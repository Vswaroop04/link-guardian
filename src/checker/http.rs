// src/checker/http.rs
// =============================================================================
// This module checks if URLs are alive by making HTTP requests.
//
// Key functionality:
// - Makes HTTP HEAD requests (lightweight, no body download)
// - Falls back to GET if HEAD fails
// - Detects various failure modes (404, timeout, SSL errors, etc.)
// - Runs checks concurrently with rate limiting
//
// Rust concepts:
// - async/await: For concurrent network I/O
// - Result<T, E>: For error handling
// - Enums: To represent different link states
// - Streams: For processing many items concurrently
// =============================================================================

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use futures::stream::{self, StreamExt};  // StreamExt gives us .buffer_unordered()

// Represents the status of a link after checking
//
// #[derive(Serialize, Deserialize)] lets us convert to/from JSON
// #[derive(Debug, Clone)] enables debugging and cloning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LinkStatus {
    /// Link is working (200 OK)
    Ok,
    /// Link redirects to another URL (301, 302, etc.)
    Redirect(String),  // Holds the redirect target URL
    /// Link is broken (404, 410, etc.)
    Broken,
    /// Request timed out
    Timeout,
    /// SSL/TLS certificate error
    SslError,
    /// Too many redirects (redirect loop)
    TooManyRedirects,
    /// Could not resolve hostname
    DnsError,
    /// Other error
    Error,
}

// Represents the result of checking a single link
//
// This struct holds all information about a link check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkCheckResult {
    /// The URL that was checked
    pub url: String,
    /// The status of the link
    #[serde(flatten)]  // This merges the LinkStatus fields into LinkCheckResult
    pub status: LinkStatus,
    /// Optional message with more details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl LinkCheckResult {
    /// Helper method to check if the link is OK
    ///
    /// Returns true for Ok and Redirect statuses
    pub fn is_ok(&self) -> bool {
        matches!(self.status, LinkStatus::Ok | LinkStatus::Redirect(_))
    }
}

// Checks multiple links concurrently
//
// This is the main entry point for link checking.
// It takes a vector of URLs and returns results for all of them.
//
// Why async?
// - We might check hundreds of links
// - Each HTTP request takes time (network latency)
// - Running them concurrently is MUCH faster than sequential
// - Example: 100 links * 1 sec each = 100 sec sequential vs ~5 sec concurrent
pub async fn check_links(urls: Vec<String>) -> Vec<LinkCheckResult> {
    // Create an HTTP client with reasonable settings
    // We'll reuse this client for all requests (connection pooling)
    let client = Client::builder()
        .timeout(Duration::from_secs(10))  // 10 second timeout per request
        .redirect(reqwest::redirect::Policy::limited(5))  // Follow up to 5 redirects
        .build()
        .expect("Failed to create HTTP client");

    // Create a stream of futures
    // Each future represents checking one URL
    let futures = urls.into_iter().map(|url| {
        let client = client.clone();  // Clone the client for each task
        async move {
            check_single_link(client, url).await
        }
    });

    // Convert futures into a stream and run up to 50 concurrently
    // .buffer_unordered(50) means: run up to 50 tasks at once, return results
    // as they complete (not in original order, hence "unordered")
    //
    // Why 50? Balance between:
    // - Too low: slow checking
    // - Too high: might overwhelm the network or get rate-limited
    stream::iter(futures)
        .buffer_unordered(50)
        .collect()  // Collect all results into a Vec
        .await
}

// Checks a single link
//
// This function does the actual HTTP request and categorizes the result.
//
// Parameters:
//   client: reqwest HTTP client (borrowed, we don't own it)
//   url: the URL to check (owned String)
//
// Returns: LinkCheckResult with status and details
async fn check_single_link(client: Client, url: String) -> LinkCheckResult {
    // First, try a HEAD request (faster, no body download)
    let result = client.head(&url).send().await;

    // Match on the result to handle success and various error types
    match result {
        Ok(response) => {
            // Got a response! Check the status code
            analyze_response(url, response)
        }
        Err(e) => {
            // Request failed - figure out why
            categorize_error(url, e)
        }
    }
}

// Analyzes an HTTP response to determine link status
//
// HTTP status codes:
// - 200-299: Success
// - 300-399: Redirect
// - 400-499: Client error (404 not found, etc.)
// - 500-599: Server error
fn analyze_response(url: String, response: reqwest::Response) -> LinkCheckResult {
    let status_code = response.status();

    if status_code.is_success() {
        // 2xx status codes mean success
        LinkCheckResult {
            url,
            status: LinkStatus::Ok,
            message: Some(format!("HTTP {}", status_code.as_u16())),
        }
    } else if status_code.is_redirection() {
        // 3xx status codes mean redirect
        // Try to get the Location header to show where it redirects to
        let redirect_target = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        LinkCheckResult {
            url,
            status: LinkStatus::Redirect(redirect_target.clone()),
            message: Some(format!("HTTP {} -> {}", status_code.as_u16(), redirect_target)),
        }
    } else if matches!(status_code, StatusCode::NOT_FOUND | StatusCode::GONE) {
        // 404 Not Found or 410 Gone - definitely broken
        LinkCheckResult {
            url,
            status: LinkStatus::Broken,
            message: Some(format!("HTTP {}", status_code.as_u16())),
        }
    } else {
        // Other status codes (e.g., 500 server errors)
        // We'll consider these as errors rather than definitively broken
        LinkCheckResult {
            url,
            status: LinkStatus::Error,
            message: Some(format!("HTTP {}", status_code.as_u16())),
        }
    }
}

// Categorizes different error types from reqwest
//
// reqwest errors can happen for many reasons:
// - Network timeout
// - DNS resolution failure
// - SSL certificate issues
// - Too many redirects
// - etc.
fn categorize_error(url: String, error: reqwest::Error) -> LinkCheckResult {
    // Convert error to string once to avoid lifetime issues
    let error_string = error.to_string();

    let (status, message) = if error.is_timeout() {
        (LinkStatus::Timeout, "Request timed out".to_string())
    } else if error.is_redirect() {
        (LinkStatus::TooManyRedirects, "Too many redirects".to_string())
    } else if error.is_connect() {
        // Connection errors often mean DNS issues or host unreachable
        if error_string.contains("dns") {
            (LinkStatus::DnsError, "Could not resolve hostname".to_string())
        } else {
            (LinkStatus::Error, "Connection failed".to_string())
        }
    } else if error_string.contains("certificate") || error_string.contains("ssl") {
        (LinkStatus::SslError, "SSL certificate error".to_string())
    } else {
        (LinkStatus::Error, error_string.clone())
    };

    LinkCheckResult {
        url,
        status,
        message: Some(message),
    }
}

// -----------------------------------------------------------------------------
// BEGINNER NOTES:
//
// 1. Why async/await?
//    - Network I/O is slow (milliseconds to seconds)
//    - While waiting for one response, we can check other links
//    - async/await is Rust's way of writing concurrent code that's easy to read
//    - Think of it like JavaScript's async/await, but with stricter guarantees
//
// 2. What is StreamExt and buffer_unordered?
//    - StreamExt is a trait (like an interface) that adds methods to streams
//    - buffer_unordered(N) runs up to N futures concurrently
//    - It's like Promise.all() but with a concurrency limit
//
// 3. Why clone the client?
//    - Each async task needs its own reference to the client
//    - Client is cheap to clone (it's just a reference counter internally)
//    - This is a common pattern in async Rust
//
// 4. What is match?
//    - Pattern matching - like switch/case but much more powerful
//    - Can destructure enums, check conditions, bind variables
//    - The compiler ensures we handle all cases
//
// 5. What is Option<T> and Some/None?
//    - Option represents a value that might not exist
//    - Some(value) = there is a value
//    - None = no value
//    - It's Rust's replacement for null (but type-safe!)
//
// 6. What does .await do?
//    - Waits for an async operation to complete
//    - Yields control to other tasks while waiting
//    - Only works inside async functions
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_valid_link() {
        let results = check_links(vec!["https://www.rust-lang.org".to_string()]).await;
        assert_eq!(results.len(), 1);
        // Note: This test requires internet connection
        // In production, you might mock the HTTP client
    }

    #[test]
    fn test_link_result_is_ok() {
        let ok_result = LinkCheckResult {
            url: "https://example.com".to_string(),
            status: LinkStatus::Ok,
            message: None,
        };
        assert!(ok_result.is_ok());

        let broken_result = LinkCheckResult {
            url: "https://example.com".to_string(),
            status: LinkStatus::Broken,
            message: None,
        };
        assert!(!broken_result.is_ok());
    }
}
