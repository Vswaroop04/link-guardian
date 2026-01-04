# link-guardian 🔗🛡️

A beginner-friendly Rust CLI tool to scan GitHub repositories and websites for broken links.

Perfect for:
- CI/CD pipelines to catch broken documentation links
- Maintaining website link health
- Learning Rust while building a practical tool

## Features

- ✅ Scan GitHub repositories (README.md)
- ✅ Scan websites with configurable crawl depth
- ✅ Detect broken links (404, timeouts, SSL errors, etc.)
- ✅ Detect redirects (301, 302)
- ✅ Human-readable table output
- ✅ JSON output for scripting/CI
- ✅ Proper exit codes for CI integration
- ✅ Concurrent link checking (fast!)
- ✅ Polite crawling with delays

## Installation

### Prerequisites

- Rust 1.70 or newer ([install from rust-lang.org](https://www.rust-lang.org/tools/install))

### Build from source

```bash
# Clone the repository (or navigate to the project folder)
cd link-guardian

# Build in release mode (optimized)
cargo build --release

# The binary will be at target/release/link-guardian
```

### Install globally (optional)

```bash
cargo install --path .

# Now you can run 'link-guardian' from anywhere
```

## Usage

### Basic Commands

#### Scan a GitHub repository

```bash
# Check links in a GitHub repo's README
link-guardian github https://github.com/rust-lang/rust

# With JSON output
link-guardian github https://github.com/rust-lang/rust --json
```

#### Scan a website

```bash
# Scan just the homepage
link-guardian site https://example.com

# Scan homepage + all linked pages (depth 2)
link-guardian site https://example.com --max-depth 2

# With JSON output
link-guardian site https://example.com --json
```

### Command-line Options

```
link-guardian --help

Commands:
  github  Scan a GitHub repository for broken links
  site    Scan a website for broken links
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

#### GitHub subcommand

```
link-guardian github --help

Scan a GitHub repository for broken links in README and docs/

Usage: link-guardian github [OPTIONS] <REPO_URL>

Arguments:
  <REPO_URL>  GitHub repository URL (e.g., https://github.com/user/repo)

Options:
      --json  Output results in JSON format instead of a table
  -h, --help  Print help
```

#### Site subcommand

```
link-guardian site --help

Scan a website for broken links

Usage: link-guardian site [OPTIONS] <WEBSITE_URL>

Arguments:
  <WEBSITE_URL>  Website URL to scan (e.g., https://example.com)

Options:
      --json              Output results in JSON format instead of a table
      --max-depth <MAX_DEPTH>  Maximum crawl depth (default: 1) [default: 1]
  -h, --help              Print help
```

## Output Examples

### Table Output (default)

```
🔍 Scanning website: https://example.com
📊 Max crawl depth: 1
📄 Crawled 1 page(s)
   5 links found on https://example.com

🌐 Checking 5 unique link(s)...

URL                                                          STATUS          MESSAGE
=========================================================================================================
https://example.com/about                                    ✅ OK           HTTP 200
https://example.com/contact                                  ✅ OK           HTTP 200
https://example.com/old-page                                 🔀 REDIRECT     HTTP 301 -> /new-page
https://example.com/missing                                  ❌ BROKEN       HTTP 404
https://example.com/timeout                                  ⏱️  TIMEOUT     Request timed out

📊 Summary:
   ✅ OK: 2
   ❌ Broken: 3
   📋 Total: 5
```

### JSON Output

```bash
link-guardian site https://example.com --json
```

```json
[
  {
    "url": "https://example.com/about",
    "status": "ok",
    "message": "HTTP 200"
  },
  {
    "url": "https://example.com/old-page",
    "status": "redirect",
    "redirect": "https://example.com/new-page",
    "message": "HTTP 301 -> https://example.com/new-page"
  },
  {
    "url": "https://example.com/missing",
    "status": "broken",
    "message": "HTTP 404"
  }
]
```

## Exit Codes

Perfect for CI/CD integration:

- **0**: All links are OK (success)
- **1**: Broken links detected (failure)
- **2**: Internal error or invalid usage

### Example CI Usage

```bash
#!/bin/bash
# In your CI script

link-guardian github https://github.com/youruser/yourrepo

if [ $? -eq 1 ]; then
  echo "❌ Broken links detected!"
  exit 1
else
  echo "✅ All links are healthy!"
fi
```

## Project Structure

```
link-guardian/
├── Cargo.toml              # Project metadata and dependencies
├── README.md               # This file
└── src/
    ├── main.rs             # Entry point, orchestrates everything
    ├── cli.rs              # Command-line parsing (clap)
    ├── checker/
    │   ├── mod.rs          # Checker module exports
    │   ├── http.rs         # HTTP link checking logic
    │   ├── markdown.rs     # Extract links from Markdown
    │   └── html.rs         # Extract links from HTML
    ├── github/
    │   ├── mod.rs          # GitHub module exports
    │   └── fetch.rs        # Fetch files from GitHub repos
    └── crawl/
        ├── mod.rs          # Crawl module exports
        └── queue.rs        # Website crawling with BFS
```

## How It Works

### For GitHub Repositories

1. Parse the GitHub URL to extract owner/repo
2. Fetch README.md from `raw.githubusercontent.com`
3. Parse Markdown and extract all HTTP/HTTPS links
4. Check each link concurrently (up to 50 at a time)
5. Report results

### For Websites

1. Fetch the starting URL
2. Extract all links from the HTML
3. If max-depth > 1, crawl same-domain links (breadth-first)
4. Collect all unique links found across all pages
5. Check each link concurrently (up to 50 at a time)
6. Report results

### Link Checking

For each link:
- Make an HTTP HEAD request (lightweight, no body)
- Categorize the response:
  - 200-299: ✅ OK
  - 300-399: 🔀 Redirect
  - 404/410: ❌ Broken
  - Timeout: ⏱️ Timeout
  - SSL errors: 🔒 SSL Error
  - DNS errors: 🌐 DNS Error
  - Other: ⚠️ Error

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_check_valid_link
```

### Running in Development

```bash
# Run without building a release binary
cargo run -- github https://github.com/rust-lang/rust

cargo run -- site https://example.com --max-depth 2
```

### Code Style

```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy
```

## Learning Resources for Rust Beginners

The code is heavily commented to teach Rust concepts. Look for:
- **Function-level comments**: Explain what each function does
- **Inline comments**: Explain tricky Rust concepts
- **"BEGINNER NOTES" sections**: Deep dives into Rust concepts

Key Rust concepts used in this project:
- **Modules**: Organizing code into namespaces
- **async/await**: Concurrent programming for network I/O
- **Result<T, E>**: Type-safe error handling
- **Option<T>**: Representing values that might not exist
- **Ownership**: Who owns data and when it's freed
- **Borrowing**: Temporary access to data without owning it
- **Traits**: Like interfaces in other languages
- **Pattern matching**: The `match` keyword for control flow
- **Iterators**: Processing sequences of items efficiently

## Troubleshooting

### "Failed to fetch README.md"

- The repository might use `master` instead of `main` branch
- The repository might not have a README.md
- Check the URL is correct: `https://github.com/owner/repo`

### "SSL certificate error"

- Some websites have invalid or expired SSL certificates
- This is reported as a "broken" link for safety

### "Too many redirects"

- The URL might have a redirect loop
- Default limit is 5 redirects

### Rate Limiting

- GitHub's raw.githubusercontent.com has rate limits
- For heavy usage, consider implementing GitHub API with authentication
- Websites might rate-limit or block rapid requests

## Future Enhancements (Stretch Goals)

- [ ] Use GitHub API (octocrab) for better repo access
- [ ] Colored terminal output
- [ ] Progress bars for long scans
- [ ] Configurable ignore patterns (skip certain URLs)
- [ ] Support for other platforms (GitLab, Bitbucket)
- [ ] Retry logic for transient failures
- [ ] HTML report generation
- [ ] Recursive docs/ folder scanning for GitHub repos

## Contributing

This is a learning project! Contributions welcome:
1. Fork the repository
2. Create a feature branch
3. Make your changes (keep the teaching style!)
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Credits

Built with:
- [clap](https://github.com/clap-rs/clap) - Command-line parsing
- [tokio](https://github.com/tokio-rs/tokio) - Async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [scraper](https://github.com/causal-agent/scraper) - HTML parsing
- [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark) - Markdown parsing
- [url](https://github.com/servo/rust-url) - URL parsing
- [serde](https://github.com/serde-rs/serde) - Serialization
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling

---

Made with ❤️ for Rust learners
