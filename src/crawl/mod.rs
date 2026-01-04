// src/crawl/mod.rs
// =============================================================================
// This module handles website crawling.
//
// Features:
// - Breadth-first crawling starting from a URL
// - Respects same-domain restriction (doesn't crawl external sites)
// - Configurable depth limit
// - Polite crawling with delays between requests
//
// Why crawl?
// - To find all pages on a website
// - To extract all links from those pages
// - To provide comprehensive link checking
//
// Rust concepts:
// - Async programming: For concurrent network requests
// - Collections: HashSet for tracking visited URLs, VecDeque for queue
// =============================================================================

mod queue;

// Re-export the main crawling function
pub use queue::crawl_website;
