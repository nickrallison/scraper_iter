use clap::Parser;
use futures::StreamExt;
use tokio::sync::mpsc;

mod crawler;

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

    /// Filter pattern to decide whether to proceed with a URL's children
    #[arg(short, long, default_value = "")]
    filter_pattern: String,

    /// Site to perform site search and add URLs from
    #[arg(long)]
    search_site: Option<String>,
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
    let filter_pattern = args.filter_pattern;
    let filter = move |url: &String| url.contains(&filter_pattern);

    // Create a channel for dynamically added URLs
    let (url_sender, url_receiver) = mpsc::unbounded_channel::<String>();

    // If search_site is specified, start the search task
    if let Some(search_site) = args.search_site.clone() {
        // Clone the sender to move into the async task
        let sender_clone = url_sender.clone();
        tokio::spawn(async move {
            search_site_urls(&search_site, sender_clone).await;
        });
    }

    // Send initial URLs into the sender
    for url in initial_urls {
        url_sender
            .send(url)
            .unwrap_or_else(|err| eprintln!("Error sending initial URL: {}", err));
    }

    // Start crawling and get the stream of URLs
    let mut stream = crawler::crawl_urls(url_receiver, filter);

    // Process the stream of URLs
    while let Some(url) = stream.next().await {
        println!("{}", url);
    }
}

/// Performs a site-specific internet search using the Searx search engine API.
/// Found URLs are sent to the crawler via the provided sender.
async fn search_site_urls(search_site: &String, url_sender: mpsc::UnboundedSender<String>) {
    let mut page = 1;
    loop {
        let query = format!("site:{}", search_site);
        let url = format!(
            "https://searx.be/search?q={}&format=json&safesearch=1&engines=google&language=en-US&page={}",
            urlencoding::encode(&query),
            page
        );
        let resp = match reqwest::get(&url).await {
            Ok(resp) => resp,
            Err(err) => {
                eprintln!("Error fetching search results: {}", err);
                break;
            }
        };

        let json: serde_json::Value = match resp.json().await {
            Ok(json) => json,
            Err(err) => {
                eprintln!("Error parsing search results JSON: {}", err);
                break;
            }
        };

        if let Some(results) = json.get("results") {
            if results.is_array() {
                let results_array = results.as_array().unwrap();
                if results_array.is_empty() {
                    // No more results
                    break;
                }
                for result in results_array {
                    if let Some(url) = result.get("url") {
                        if let Some(url_str) = url.as_str() {
                            url_sender
                                .send(url_str.to_string())
                                .unwrap_or_else(|err| eprintln!("Error sending URL: {}", err));
                        }
                    }
                }
            } else {
                // No results array
                break;
            }
        } else {
            // No results field in JSON
            break;
        }
        page += 1;
        // Wait a bit to be polite to the server and avoid rate limiting
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
}