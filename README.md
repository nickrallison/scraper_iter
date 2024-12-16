# Web Crawler

This project is a simple and asynchronous web crawler written in Rust. It uses `reqwest` and `scraper` to fetch and parse web pages for links, and `futures` and `tokio` to handle asynchronous operations. You can start the crawler using initial URLs, perform site-specific searches, and apply a filter to decide which links to further crawl based on specific patterns.

## Features

- **Asynchronous Crawling**: Utilizes Rust's async ecosystem to perform web crawling efficiently.
- **Link Filtering**: Allows filtering of URLs to control which pages should be explored further.
- **Site-Specific Search**: Perform searches on specific sites and add the results to the crawl queue.
- **Command-line Interface**: Uses `clap` to specify parameters through command-line arguments.

## Prerequisites

- **Rust**: Ensure you have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).

## Getting Started

### Cloning the Repository

```bash
git clone https://github.com/yourusername/web-crawler.git
cd web-crawler
```

### Building the Project

```bash
cargo build --release
```

## Usage

Run the web crawler with different configuration options using command-line arguments.

### Basic Usage

To start crawling from a single URL:

```bash
cargo run -- --url https://www.rust-lang.org
```

### Multiple URLs

Start crawling from multiple URLs:

```bash
cargo run -- --url https://www.rust-lang.org --url https://www.mozilla.org
```

### Using a File for URLs

Provide a file containing initial URLs, with each URL on a new line:

```bash
cargo run -- --input-file links.txt
```

### Custom Filter Pattern

Set a custom filter pattern to determine which URLs should have their child links crawled:

```bash
cargo run -- --url https://www.rust-lang.org --filter-pattern rust-lang.org
```

### Site-Specific Search

Perform a site-specific search and add URLs from the search results:

```bash
cargo run -- --search-site example.com --search-limit 5
```

### Output to File

Save the output links to a specified file:

```bash
cargo run -- --url https://www.rust-lang.org --output-path output.txt
```

## Command-Line Options

- `-u, --url`: Specify initial URLs to crawl from. Multiple URLs can be provided by repeating this option.
- `-i, --input-file`: Specify a file containing initial URLs.
- `-f, --filter-pattern`: Specify a pattern to filter URLs. Only URLs containing this pattern will have their children crawled.
- `--search-site`: Specify a site to perform a search and add URLs from the search results.
- `--search-limit`: Specify the maximum number of search results to retrieve. Default is 10.
- `--output-path`: Specify a file to output the found links.

## Publicly Accessible Code

The project exposes several modules and functions that can be used independently or extended for additional functionality:

### Modules

- **crawler**: Contains the logic for crawling URLs. It provides the `crawl_urls` function, which takes a receiver for URLs and a filter function to control the crawling process.
- **search**: Provides functionality for performing site-specific searches. It includes the `search_site_urls` function, which performs a search and sends found URLs to a specified channel, and `search_site_urls_stream`, which returns a stream of found URLs.

### Functions

- **crawl_urls**: This function takes an `UnboundedReceiver<String>` for URLs and a filter function. It returns an asynchronous stream of URLs as they are found. It allows filtering of URLs, and only if a URL is not filtered out, its children are crawled.
- **search_site_urls**: Performs a site-specific internet search and sends found URLs to the crawler via the provided sender. It takes the site to search, the maximum number of search results to retrieve, and a sender to send found URLs.
- **search_site_urls_stream**: Similar to `search_site_urls`, but returns a stream of found URLs instead of sending them through a channel. It is useful for integrating search results directly into other asynchronous workflows.

## Example Input File (links.txt)

```
https://www.rust-lang.org
https://www.mozilla.org
```

## License

This project is licensed under the MIT License. See the LICENSE file for more information.

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/your-feature`).
3. Commit your changes (`git commit -am 'Add some feature'`).
4. Push to the branch (`git push origin feature/your-feature`).
5. Create a new Pull Request.

## Contact

For questions or any discussion, please open an issue on this repository.