// src/checker/mod.rs
// =============================================================================
// This module contains all link checking logic.
//
// Submodules:
// - http: Makes HTTP requests to check if links are alive
// - markdown: Extracts links from markdown text
// - html: Extracts links from HTML pages
//
// This file (mod.rs) is the module root - it ties everything together and
// exports the public API that other parts of our application can use.
//
// Rust concepts:
// - Modules: Organize code into namespaces
// - pub use: Re-export items to simplify imports for users of this module
// - async: Asynchronous code that can run concurrently
// =============================================================================

// Declare submodules (tells Rust to include these files)
mod http;
mod markdown;
mod html;

// Re-export public items from submodules
// This lets users write `checker::check_links()` instead of
// `checker::http::check_links()`
pub use http::{check_links, LinkCheckResult, LinkStatus};
pub use markdown::extract_markdown_links;
pub use html::extract_html_links;

// -----------------------------------------------------------------------------
// BEGINNER NOTES:
//
// 1. What is mod.rs?
//    - When you have a directory as a module (like src/checker/), the
//      mod.rs file inside it is the module root
//    - It's like index.js in JavaScript or __init__.py in Python
//
// 2. Why use 'pub use'?
//    - It re-exports items from submodules
//    - Makes the API cleaner for users of this module
//    - They don't need to know about our internal organization
//
// 3. Module privacy:
//    - By default, modules are private
//    - We explicitly choose what to make public with 'pub'
//    - This gives us control over our API surface
// -----------------------------------------------------------------------------
