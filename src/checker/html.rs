// src/checker/html.rs
// =============================================================================
// This module extracts links from HTML pages.
//
// We use the `scraper` crate which:
// - Parses HTML into a DOM (Document Object Model)
// - Supports CSS selectors for finding elements
// - Is built on html5ever (Mozilla's HTML parser)
//
// We also use the `url` crate to:
// - Parse and validate URLs
// - Resolve relative URLs to absolute URLs
//
// Rust concepts:
// - Result<T, E>: For operations that can fail
// - Iterators: For processing collections
// - Closures: Anonymous functions (|x| ...)
// =============================================================================

use scraper::{Html, Selector};
use url::Url;

// Extracts all links from HTML content
//
// Parameters:
//   html: the HTML content to parse (borrowed as &str)
//   base_url: the URL of the page (for resolving relative links)
//
// Returns: Vec<String> containing all absolute URLs found
//
// Example:
//   html = "<a href='/docs'>Docs</a>"
//   base_url = "https://example.com"
//   result = ["https://example.com/docs"]
pub fn extract_html_links(html: &str, base_url: &str) -> Vec<String> {
    let mut links = Vec::new();

    // Parse the HTML into a document
    let document = Html::parse_document(html);

    // Create a CSS selector to find all <a> tags
    // Selector::parse returns Result, so we use .unwrap() which panics on error
    // This is OK here because our selector is a constant and known to be valid
    let selector = Selector::parse("a[href]").unwrap();

    // Parse the base URL once
    // We'll use this to resolve relative links
    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => {
            // If base URL is invalid, we can't resolve relative links
            eprintln!("Warning: Invalid base URL: {}", base_url);
            return links;
        }
    };

    // Select all <a> elements with href attributes
    for element in document.select(&selector) {
        // Get the href attribute value
        if let Some(href) = element.value().attr("href") {
            // Try to convert this to an absolute URL
            if let Some(absolute_url) = resolve_url(&base, href) {
                // Only keep HTTP/HTTPS links
                if is_checkable_link(&absolute_url) {
                    links.push(absolute_url);
                }
            }
        }
    }

    links
}

// Resolves a possibly-relative URL to an absolute URL
//
// Parameters:
//   base: the base URL (the current page)
//   href: the href value (might be relative, might be absolute)
//
// Returns: Some(absolute_url) or None if invalid
//
// Examples:
//   base = "https://example.com/page"
//   href = "/docs" -> Some("https://example.com/docs")
//   href = "../other" -> Some("https://example.com/other")
//   href = "https://other.com" -> Some("https://other.com")
//   href = "javascript:void(0)" -> None (not HTTP)
fn resolve_url(base: &Url, href: &str) -> Option<String> {
    // Try to parse href as a URL
    // If it's already absolute (has a scheme), this works
    // If it's relative, this fails, so we join it with base
    match Url::parse(href) {
        Ok(url) => Some(url.to_string()),
        Err(_) => {
            // Likely a relative URL, try joining with base
            match base.join(href) {
                Ok(url) => Some(url.to_string()),
                Err(_) => None,  // Invalid URL, skip it
            }
        }
    }
}

// Checks if a URL should be checked
//
// We skip:
// - mailto: links (email)
// - tel: links (phone)
// - javascript: links
// - data: links (inline data)
// - file: links (local files)
fn is_checkable_link(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

// -----------------------------------------------------------------------------
// BEGINNER NOTES:
//
// 1. What is scraper and how does it work?
//    - scraper parses HTML into a tree structure (DOM)
//    - You can then query it using CSS selectors (like jQuery or querySelector)
//    - "a[href]" means "all <a> tags that have an href attribute"
//
// 2. What is the url crate?
//    - Handles URL parsing and manipulation
//    - Url::parse() parses a string into a Url struct
//    - url.join() resolves relative URLs (like a browser does)
//    - Example: "https://example.com" + "../other" = "https://example.com/other"
//
// 3. Why Option<String> return type?
//    - Some URLs might be invalid or unparseable
//    - Returning Option lets us represent "no valid URL"
//    - Callers can use if let Some(...) to handle valid URLs
//
// 4. What does .value() do?
//    - element is an ElementRef (reference to an HTML element)
//    - .value() gets the underlying Element
//    - .attr("href") gets the value of the href attribute
//
// 5. What is eprintln!?
//    - Like println! but prints to stderr instead of stdout
//    - Used for warnings and errors
//    - Won't mess up JSON output on stdout
//
// 6. Why unwrap() on the selector?
//    - Selector::parse can fail if the CSS selector is invalid
//    - Our selector "a[href]" is constant and known to be valid
//    - If it fails, the program should panic (programmer error)
//    - Generally avoid unwrap() on user input!
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_absolute_link() {
        let html = r#"<a href="https://www.rust-lang.org">Rust</a>"#;
        let links = extract_html_links(html, "https://example.com");
        assert_eq!(links, vec!["https://www.rust-lang.org/"]);
    }

    #[test]
    fn test_resolve_relative_link() {
        let html = r#"<a href="/docs">Docs</a>"#;
        let links = extract_html_links(html, "https://example.com/page");
        assert_eq!(links, vec!["https://example.com/docs"]);
    }

    #[test]
    fn test_skip_mailto() {
        let html = r#"<a href="mailto:test@example.com">Email</a>"#;
        let links = extract_html_links(html, "https://example.com");
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_multiple_links() {
        let html = r#"
            <a href="https://rust-lang.org">Rust</a>
            <a href="/docs">Docs</a>
            <a href="../about">About</a>
        "#;
        let links = extract_html_links(html, "https://example.com/page/");
        assert_eq!(links.len(), 3);
    }
}
