// main.rs
use std::fs::File;
use clap::Parser;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio::io::{AsyncWriteExt, BufWriter};

mod crawler;
mod search; // New module for search functionality

/// Simple web crawler to find URLs starting from initial links or search results.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Initial URLs to start crawling from
    #[arg(short, long)]
    url: Vec<String>,

    /// File containing initial URLs to start crawling from
    #[arg(short, long)]
    input_file: Option<String>,

    /// Filter pattern to decide whether to proceed with a URL's children, eahc pattern is ORed together
    #[arg(short, long)]
    filter_pattern: Vec<String>,

    /// Site to perform site search and add URLs from
    #[arg(long)]
    search_site: Option<String>,

    /// Maximum number of search results to retrieve
    #[arg(long, default_value_t = 10)]
    search_limit: u32,

    /// If given, output links will be output to given file
    #[arg(long)]
    output_path: Option<String>,

    /// If given the found links will be downloaded with wget
    #[arg(long)]
    wget: bool
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Collect initial URLs from arguments or input file
    let mut initial_urls = args.url.clone();
    if let Some(filepath) = &args.input_file {
        let file_content = tokio::fs::read_to_string(filepath)
            .await
            .expect("Failed to read input file");
        initial_urls.extend(
            file_content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty()),
        );
    }
    // Check if there are any starting points
    if initial_urls.is_empty() && args.search_site.is_none() {
        println!("No initial URLs provided. Use --url, --input-file, or --search-site to specify starting URLs.");
        return;
    }
    // Define the filter function based on the filter pattern
    let filter_patterns: Vec<String> = args.filter_pattern.clone();

    let filter = move |url: &String| {
        filter_patterns.iter().any(|pattern| url.contains(pattern))
    };

    // Create a channel for dynamically added URLs
    let (url_sender, url_receiver) = mpsc::unbounded_channel::<String>();

    // If search_site is specified, start the search task
    if let Some(search_site) = args.search_site.clone() {
        // Clone the sender to move into the async task
        let sender_clone = url_sender.clone();
        let search_limit = args.search_limit;
        tokio::spawn(async move {
            // Perform the site-specific search
            if let Err(err) =
                search::search_site_urls(&search_site, search_limit, sender_clone).await
            {
                eprintln!("Error during site search: {}", err);
            }
        });
    }
    // Send initial URLs into the sender
    for url in initial_urls {
        url_sender
            .send(url)
            .unwrap_or_else(|err| eprintln!("Error sending initial URL: {}", err));
    }
    // Open the output file if specified
    let mut output_file: Option<BufWriter<tokio::fs::File>> = match &args.output_path {
        None => None,
        Some(path) => {
            match tokio::fs::File::create(path).await {
                Ok(f) => Some(BufWriter::new(f)),
                Err(err) => {
                    eprintln!("Error creating output file: {}", err);
                    return;
                }
            }
        }
    };

    // Start crawling and get the stream of URLs
    let mut stream = crawler::crawl_urls(url_receiver, filter.clone());

    // Process the stream of URLs
    while let Some(url) = stream.next().await {
        // Clone the URL for the async task
        let url_for_task = url.clone();

        // wgetting
        if args.wget && filter(&url_for_task) {
            tokio::spawn(async move {
                if let Err(err) = crawler::wget(&url_for_task).await {
                    eprintln!("Error wgetting: {}", err);
                }
            });
        }

        // file writing
        if let Some(writer) = &mut output_file {
            writer.write_all(url.as_bytes()).await.unwrap();
            writer.write_all(b"\n").await.unwrap();
        } else {
            println!("{}", &url);
        }
    }
    // Flush the output file if necessary
    if let Some(writer) = &mut output_file {
        writer.flush().await.unwrap();
    }
}