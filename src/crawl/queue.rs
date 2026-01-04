// src/crawl/queue.rs
// =============================================================================
// This module implements website crawling with a breadth-first approach.
//
// How it works:
// 1. Start with the initial URL in a queue
// 2. Fetch the page HTML
// 3. Extract all links from the page
// 4. Add same-domain links to the queue (if not visited and within depth limit)
// 5. Repeat until queue is empty or max depth reached
//
// Politeness:
// - Adds delay between requests to avoid overwhelming servers
// - Only crawls same domain to respect boundaries
//
// Rust concepts:
// - HashSet: To track visited URLs (O(1) lookup)
// - VecDeque: Double-ended queue for breadth-first crawling
// - Url: For parsing and comparing domains
// =============================================================================

use anyhow::{anyhow, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::{HashSet, VecDeque};
use url::Url;

// Represents a page in the crawl queue
#[derive(Debug, Clone)]
struct CrawlItem {
    url: String,
    depth: usize,  // How many levels deep from the starting URL
}

// Crawls a website starting from a URL
//
// Parameters:
//   start_url: The URL to start crawling from
//   max_depth: Maximum crawl depth (1 = just the starting page)
//
// Returns: Vec of (url, html_content) tuples for all crawled pages
//
// Example:
//   max_depth=1: Only crawl the starting page
//   max_depth=2: Crawl starting page + all pages it links to
//   max_depth=3: ... + all pages those link to
pub async fn crawl_website(start_url: &str, max_depth: usize) -> Result<Vec<(String, String)>> {
    // Parse and validate the starting URL
    let start = Url::parse(start_url)
        .map_err(|e| anyhow!("Invalid URL '{}': {}", start_url, e))?;

    // Extract the domain from the starting URL
    // We'll only crawl pages on this domain
    let base_domain = start.domain()
        .ok_or_else(|| anyhow!("URL has no domain: {}", start_url))?;

    // Create HTTP client
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Queue of pages to crawl
    // VecDeque allows efficient push/pop from both ends
    let mut queue = VecDeque::new();
    queue.push_back(CrawlItem {
        url: start_url.to_string(),
        depth: 1,
    });

    // Track visited URLs to avoid crawling the same page twice
    let mut visited = HashSet::new();

    // Store results: (url, html_content)
    let mut results = Vec::new();

    // Process the queue until empty
    while let Some(item) = queue.pop_front() {
        // Skip if already visited
        if visited.contains(&item.url) {
            continue;
        }

        // Mark as visited
        visited.insert(item.url.clone());

        println!("  Crawling [depth {}]: {}", item.depth, item.url);

        // Fetch the page
        match fetch_page(&client, &item.url).await {
            Ok(html) => {
                // Store the result
                results.push((item.url.clone(), html.clone()));

                // If we haven't reached max depth, extract links and add to queue
                if item.depth < max_depth {
                    let links = extract_same_domain_links(&html, &item.url, base_domain);

                    for link in links {
                        // Only add if not visited
                        if !visited.contains(&link) {
                            queue.push_back(CrawlItem {
                                url: link,
                                depth: item.depth + 1,
                            });
                        }
                    }
                }

                // Polite crawling: small delay between requests
                // This avoids overwhelming the server
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                eprintln!("  Warning: Failed to fetch {}: {}", item.url, e);
            }
        }
    }

    Ok(results)
}

// Fetches a web page and returns its HTML content
async fn fetch_page(client: &Client, url: &str) -> Result<String> {
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("HTTP {}", response.status()));
    }

    let html = response.text().await?;
    Ok(html)
}

// Extracts links from HTML that are on the same domain
//
// This prevents the crawler from leaving the target website
//
// Parameters:
//   html: The HTML content to parse
//   page_url: The URL of the current page (for resolving relative links)
//   base_domain: The domain we're restricting crawling to
//
// Returns: Vec of absolute URLs on the same domain
fn extract_same_domain_links(html: &str, page_url: &str, base_domain: &str) -> Vec<String> {
    let mut links = Vec::new();

    // Parse the HTML
    let document = Html::parse_document(html);

    // Select all <a> tags with href
    let selector = Selector::parse("a[href]").unwrap();

    // Parse the page URL for resolving relative links
    let base = match Url::parse(page_url) {
        Ok(url) => url,
        Err(_) => return links,
    };

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            // Try to resolve to absolute URL
            let absolute_url = match resolve_link(&base, href) {
                Some(url) => url,
                None => continue,
            };

            // Check if it's on the same domain
            if let Ok(parsed) = Url::parse(&absolute_url) {
                // Only include if:
                // 1. It's HTTP/HTTPS
                // 2. It's on the same domain
                if (parsed.scheme() == "http" || parsed.scheme() == "https")
                    && parsed.domain() == Some(base_domain)
                {
                    links.push(absolute_url);
                }
            }
        }
    }

    links
}

// Resolves a link (possibly relative) to an absolute URL
fn resolve_link(base: &Url, href: &str) -> Option<String> {
    // Skip anchors and special protocols
    if href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("javascript:")
    {
        return None;
    }

    // Try to resolve the URL
    match base.join(href) {
        Ok(url) => Some(url.to_string()),
        Err(_) => None,
    }
}

// -----------------------------------------------------------------------------
// BEGINNER NOTES:
//
// 1. What is VecDeque?
//    - A double-ended queue (deck)
//    - Can efficiently add/remove from both front and back
//    - Perfect for breadth-first search (BFS)
//    - push_back() adds to end, pop_front() removes from start
//
// 2. What is HashSet?
//    - A set of unique items (no duplicates)
//    - Very fast lookup: O(1) to check if item exists
//    - We use it to track visited URLs
//    - Like Set in JavaScript or set in Python
//
// 3. What is while let?
//    - Loop while pattern matching succeeds
//    - while let Some(item) = queue.pop_front() means:
//      "while there's an item in the queue, bind it to 'item' and loop"
//    - Stops when queue.pop_front() returns None (empty queue)
//
// 4. Why depth tracking?
//    - Prevents infinite crawling
//    - Limits scope to nearby pages
//    - Each level = one more link hop from start
//
// 5. What is tokio::time::sleep?
//    - Async sleep (doesn't block the thread)
//    - Allows other tasks to run while waiting
//    - Used for polite crawling (delay between requests)
//
// 6. Why clone()?
//    - url and html are owned by item/local variables
//    - We need to store them in results (which lives longer)
//    - clone() creates a copy so we can keep both
//    - In Rust, you can't have two owners without cloning
//
// 7. What does .domain() return?
//    - Option<&str> containing the domain name
//    - Some("example.com") for http://example.com/path
//    - None for URLs without domains (like file://)
//
// 8. Breadth-first vs depth-first:
//    - Breadth-first: Crawl all pages at depth 1, then depth 2, etc.
//    - Depth-first: Follow one path all the way down, then backtrack
//    - We use breadth-first (VecDeque) for more balanced crawling
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_absolute_link() {
        let base = Url::parse("https://example.com/page").unwrap();
        let result = resolve_link(&base, "https://other.com");
        assert_eq!(result, Some("https://other.com/".to_string()));
    }

    #[test]
    fn test_resolve_relative_link() {
        let base = Url::parse("https://example.com/page").unwrap();
        let result = resolve_link(&base, "/docs");
        assert_eq!(result, Some("https://example.com/docs".to_string()));
    }

    #[test]
    fn test_skip_anchor() {
        let base = Url::parse("https://example.com/page").unwrap();
        let result = resolve_link(&base, "#section");
        assert_eq!(result, None);
    }

    #[test]
    fn test_skip_mailto() {
        let base = Url::parse("https://example.com/page").unwrap();
        let result = resolve_link(&base, "mailto:test@example.com");
        assert_eq!(result, None);
    }
}
