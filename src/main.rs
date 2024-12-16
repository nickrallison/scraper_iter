mod error;

use async_stream::stream;
use futures::Stream;
use futures::StreamExt;
use reqwest;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::fs::read_to_string;

/// Crawls the given initial URLs and returns an asynchronous stream of found URLs.
/// It allows filtering of URLs, and only if a URL is not filtered out, its children are crawled.
///
/// # Arguments
///
/// * `initial_urls` - A vector of initial URLs to start crawling from.
/// * `filter` - A function that takes a URL and returns a boolean indicating whether to proceed with its children.
///
/// # Returns
///
/// An asynchronous stream of URLs as they are found.
fn crawl_urls<F>(
    initial_urls: Vec<String>,
    filter: F,
) -> impl Stream<Item = String>
where
    F: Fn(&String) -> bool + Send + Sync + 'static,
{
    let filter = Arc::new(filter);

    stream! {
        use futures::stream::FuturesUnordered;
        let mut visited = HashSet::new();
        let mut to_crawl = FuturesUnordered::new();

        // Start by adding the initial URLs to the crawl queue
        for url in initial_urls {
            if !visited.contains(&url) {
                visited.insert(url.clone());
                to_crawl.push(fetch_url(url));
            }
        }

        // Process the crawl queue
        while let Some((url, child_urls)) = to_crawl.next().await {
            // Yield the current URL
            yield url.clone();

            // Decide whether to proceed with the children based on the filter
            if !filter(&url) {
                continue;
            }

            // Schedule the child URLs to be crawled
            for child_url in child_urls {
                if !visited.contains(&child_url) {
                    visited.insert(child_url.clone());
                    to_crawl.push(fetch_url(child_url));
                }
            }
        }
    }
}

/// Fetches the content of the given URL and returns a tuple of the URL and its child URLs.
///
/// # Arguments
///
/// * `url` - The URL to fetch.
///
/// # Returns
///
/// A tuple containing the URL and a vector of its child URLs.
async fn fetch_url(url: String) -> (String, Vec<String>) {
    // Attempt to fetch the URL content
    let body = match reqwest::get(&url).await {
        Ok(resp) => match resp.text().await {
            Ok(body) => body,
            Err(_) => return (url, vec![]),
        },
        Err(_) => return (url, vec![]),
    };

    // Parse the content to find child links
    let document = Html::parse_document(&body);
    let selector = Selector::parse("a[href]").unwrap();
    let mut child_urls = Vec::new();
    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            let child_url = resolve_url(href, &url);
            child_urls.push(child_url);
        }
    }

    (url, child_urls)
}

/// Resolves a relative URL to an absolute URL based on the base URL.
///
/// # Arguments
///
/// * `href` - The relative or absolute href from the link.
/// * `base_url` - The base URL to resolve relative URLs.
///
/// # Returns
///
/// A string representing the absolute URL.
fn resolve_url(href: &str, base_url: &str) -> String {
    match reqwest::Url::parse(base_url) {
        Ok(base) => match base.join(href) {
            Ok(url) => url.to_string(),
            Err(_) => href.to_string(),
        },
        Err(_) => href.to_string(),
    }
}

const LINKS_FILEPATH: &str = "links.txt";

#[tokio::main]
async fn main() {
    // Specify the initial URLs to crawl
    let initial_urls: Vec<String> = read_to_string(LINKS_FILEPATH).await.unwrap().lines().map(|line| line.to_string()).collect();
    // Define a filter function to decide whether to crawl a URL's children
    let filter = |url: &String| {
        // Only proceed with URLs containing "example.com"
        url.contains("cspages.ucalgary.ca")
    };

    // Start crawling and get the stream of URLs
    let mut stream = Box::pin(crawl_urls(initial_urls, filter));

    // Process the stream of URLs
    while let Some(url) = stream.next().await {
        println!("Found URL: {}", url);

        // Here, you can perform additional processing or filtering if needed
    }
}