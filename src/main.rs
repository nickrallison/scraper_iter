use clap::Parser;
use futures::StreamExt;

mod crawler;

/// Simple web crawler to find URLs starting from initial links.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Initial URLs to start crawling from
    #[arg(short, long)]
    url: Vec<String>,

    /// File containing initial URLs to start crawling from
    #[arg(short, long)]
    input_file: Option<String>,

    /// Filter pattern to decide whether to proceed with a URL's children
    #[arg(short, long, default_value = "")]
    filter_pattern: String,
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

    if initial_urls.is_empty() {
        println!("No initial URLs provided. Use --url or --input-file to specify starting URLs.");
        return;
    }

    // Define the filter function based on the filter pattern
    let filter_pattern = args.filter_pattern;
    let filter = move |url: &String| url.contains(&filter_pattern);

    // Start crawling and get the stream of URLs
    let mut stream = crawler::crawl_urls(initial_urls, filter);

    // Process the stream of URLs
    while let Some(url) = stream.next().await {
        println!("{}", url);
    }
}