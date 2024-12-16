# Web Crawler

This project is a simple and asynchronous web crawler written in Rust. It uses `reqwest` and `scraper` to fetch and parse web pages for links, and `futures` and `tokio` to handle asynchronous operations. You can start the crawler using initial URLs and apply a filter to decide which links to further crawl based on specific patterns.

## Features

- **Asynchronous Crawling**: Utilizes Rust's async ecosystem to perform web crawling efficiently.
- **Link Filtering**: Allows filtering of URLs to control which pages should be explored further.
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

To start crawling from a single URL with the default filter pattern (`example.com`):

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

## Command-Line Options

- `-u, --url`: Specify initial URLs to crawl from. Multiple URLs can be provided by repeating this option.
- `-i, --input-file`: Specify a file containing initial URLs.
- `-f, --filter-pattern`: Specify a pattern to filter URLs. Only URLs containing this pattern will have their children crawled. Default is `example.com`.

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
