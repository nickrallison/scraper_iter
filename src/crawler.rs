// src/crawler.rs

use async_stream::stream;
use futures::stream::FuturesUnordered;
use futures::{Stream, StreamExt};
use reqwest;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;

/// Crawls URLs provided via a receiver and returns an asynchronous stream of found URLs.
/// It allows filtering of URLs, and only if a URL is not filtered out, its children are crawled.
///
/// # Arguments
///
/// * `url_receiver` - An UnboundedReceiver that provides URLs to start crawling from.
/// * `filter` - A function that takes a URL and returns a boolean indicating whether to proceed with its children.
///
/// # Returns
///
/// An asynchronous stream of URLs as they are found.
pub fn crawl_urls<F>(
    mut url_receiver: UnboundedReceiver<String>,
    filter: F,
) -> impl Stream<Item = String>
where
    F: Fn(&String) -> bool + Send + Sync + 'static,
{
    let filter = Arc::new(filter);
    let crawled_urls = Arc::new(tokio::sync::Mutex::new(HashSet::new())); // Shared set of visited URLs

    Box::pin(stream! {
        let mut to_crawl = FuturesUnordered::new();

        loop {
            tokio::select! {
                // Receive new URLs to crawl
                Some(url) = url_receiver.recv() => {
                    let mut visited = crawled_urls.lock().await;
                    if !visited.contains(&url) {
                        visited.insert(url.clone());
                        drop(visited); // Release the lock before awaiting

                        // Start fetching the URL
                        to_crawl.push(fetch_url(url));
                    }
                },
                // Process the next crawled URL
                Some((url, child_urls)) = to_crawl.next() => {
                    // Yield the current URL
                    yield url.clone();

                    // Decide whether to proceed with the children based on the filter
                    if !filter(&url) {
                        continue;
                    }

                    // Schedule the child URLs to be crawled
                    for child_url in child_urls {
                        let mut visited = crawled_urls.lock().await;
                        if !visited.contains(&child_url) {
                            visited.insert(child_url.clone());
                            drop(visited); // Release the lock before awaiting
                            to_crawl.push(fetch_url(child_url));
                        }
                    }
                },
                else => {
                    // No more URLs to receive and all crawling is done
                    break;
                }
            }
        }
    })
}

/// Fetches the content of a URL and extracts child URLs.
///
/// # Arguments
///
/// * `url` - The URL to fetch and parse.
///
/// # Returns
///
/// A tuple containing the original URL and a vector of child URLs found on the page.
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

/// Resolves relative URLs to absolute URLs based on the base URL.
///
/// # Arguments
///
/// * `href` - The href attribute value found in the link.
/// * `base_url` - The URL of the page where the link was found.
///
/// # Returns
///
/// An absolute URL as a String.
fn resolve_url(href: &str, base_url: &str) -> String {
    match reqwest::Url::parse(base_url) {
        Ok(base) => match base.join(href) {
            Ok(url) => url.to_string(),
            Err(_) => href.to_string(),
        },
        Err(_) => href.to_string(),
    }
}