// src/checker/markdown.rs
// =============================================================================
// This module extracts links from Markdown text.
//
// We use the `pulldown-cmark` crate which:
// - Parses Markdown into events (heading, paragraph, link, etc.)
// - Follows the CommonMark specification
// - Is fast and memory-efficient (it's a streaming parser)
//
// Rust concepts:
// - Iterators: For processing sequences of items
// - Pattern matching: To identify link events
// - Filtering: To skip unwanted links
// =============================================================================

use pulldown_cmark::{Parser, Event, Tag};

// Extracts all HTTP/HTTPS links from Markdown text
//
// Parameters:
//   markdown: the markdown text to parse (borrowed as &str)
//
// Returns: Vec<String> containing all the URLs found
//
// Example input:
//   "Check out [Rust](https://www.rust-lang.org)!"
//
// Example output:
//   vec!["https://www.rust-lang.org"]
pub fn extract_markdown_links(markdown: &str) -> Vec<String> {
    let mut links = Vec::new();

    // Create a Markdown parser
    // This produces an iterator of events as it parses the text
    let parser = Parser::new(markdown);

    // Track if we're currently inside a link
    // We need this because markdown parsing produces multiple events per link:
    // 1. Start(Link) - link begins
    // 2. Text - the link text
    // 3. End(Link) - link ends
    let mut current_link: Option<String> = None;

    // Iterate through all markdown events
    for event in parser {
        match event {
            // When we encounter the start of a link tag
            // In pulldown-cmark 0.9, Link is Tag::Link(link_type, dest_url, title)
            Event::Start(Tag::Link(_link_type, dest_url, _title)) => {
                // dest_url is the URL (the part in parentheses in [text](url))
                // Convert from CowStr to String
                let url = dest_url.to_string();

                // Only keep HTTP/HTTPS links
                // Skip mailto:, tel:, javascript:, etc.
                if is_http_link(&url) {
                    current_link = Some(url);
                }
            }

            // When we encounter the end of a link tag
            Event::End(Tag::Link(..)) => {
                // If we were tracking a link, add it to our results
                if let Some(url) = current_link.take() {
                    links.push(url);
                }
            }

            // We don't care about other events (text, headings, etc.)
            _ => {}
        }
    }

    links
}

// Helper function to check if a URL is an HTTP/HTTPS link
//
// We want to skip:
// - mailto: links (email addresses)
// - tel: links (phone numbers)
// - javascript: links
// - file: links
// - Relative links (will be handled differently in HTML parsing)
fn is_http_link(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

// -----------------------------------------------------------------------------
// BEGINNER NOTES:
//
// 1. What is an iterator?
//    - A way to process a sequence of items one at a time
//    - The parser returns events one by one as it reads the markdown
//    - More memory efficient than loading everything at once
//
// 2. What is Event and Tag?
//    - pulldown-cmark represents markdown as a stream of events
//    - Event::Start(Tag::Link) = a link is starting
//    - Event::End(TagEnd::Link) = a link is ending
//    - Event::Text = regular text
//    - etc.
//
// 3. What is Option<String>?
//    - Option<T> means "maybe has a value, maybe doesn't"
//    - Some(value) = has a value
//    - None = no value
//    - We use it to track whether we're currently inside a link
//
// 4. What does .take() do?
//    - Takes the value out of an Option and replaces it with None
//    - Useful for "consuming" a value while clearing the original
//
// 5. What is if let?
//    - Syntax for "if this matches a pattern, do something"
//    - if let Some(url) = current_link.take() means:
//      "if current_link has a value, bind it to url and run the block"
//
// 6. Why &str instead of String?
//    - &str is a borrowed string slice (reference)
//    - We don't need to own the markdown text, just read it
//    - More efficient - no copying needed
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_link() {
        let markdown = "Check out [Rust](https://www.rust-lang.org)!";
        let links = extract_markdown_links(markdown);
        assert_eq!(links, vec!["https://www.rust-lang.org"]);
    }

    #[test]
    fn test_extract_multiple_links() {
        let markdown = r#"
# Resources

- [Rust](https://www.rust-lang.org)
- [Cargo](https://doc.rust-lang.org/cargo/)
- [Docs](https://doc.rust-lang.org/)
        "#;
        let links = extract_markdown_links(markdown);
        assert_eq!(links.len(), 3);
        assert!(links.contains(&"https://www.rust-lang.org".to_string()));
    }

    #[test]
    fn test_skip_mailto_links() {
        let markdown = "Email me at [email](mailto:test@example.com)";
        let links = extract_markdown_links(markdown);
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_skip_relative_links() {
        let markdown = "See [docs](./docs/README.md)";
        let links = extract_markdown_links(markdown);
        assert_eq!(links.len(), 0);
    }
}
