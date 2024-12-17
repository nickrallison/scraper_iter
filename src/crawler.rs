// src/crawler.rs

use async_stream::stream;
use futures::stream::FuturesUnordered;
use futures::{Stream, StreamExt};
use reqwest;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use reqwest::Url;
use tokio::process::Command;
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
            let child_url = resolve_url(href, &url).await;
            let other_url = url.clone();
            let mut other_url = Url::parse(&other_url).expect("URL should be valid");
            {
                let mut segs = other_url.path_segments_mut().unwrap();
                segs.pop();
            }
            let other_child = resolve_url(href, other_url.as_str()).await;

            child_urls.push(child_url);
            child_urls.push(other_child);
        }
    }

    (url, child_urls)
}

/// Resolves relative URLs to absolute URLs based on the base URL
/// and checks if it is valid at runtime.
///
/// If the URL is not valid, it will attempt to resolve by trimming
/// the last part of the `base_url`.
///
/// # Arguments
///
/// * `href` - The href attribute value found in the link.
/// * `base_url` - The URL of the page where the link was found.
///
/// # Returns
///
/// An absolute URL as a String.
async fn resolve_url(href: &str, base_url: &str) -> String {
    // Attempt to parse the base URL and join with the href
    let mut resolved_url = match Url::parse(base_url).and_then(|base| base.join(href)) {
        Ok(url) => url.to_string(),
        Err(_) => return href.to_string(), // Return href if parsing fails altogether
    };

    return resolved_url;

    // Check if the resolved URL is valid
    // if is_valid_url(&resolved_url).await {
    //     return resolved_url;
    // }
    //
    // let mut url = match Url::parse(&resolved_url) {
    //     Ok(url) => url,
    //     Err(_) => return href.to_string(), // Return href if re-parsing fails
    // };
    //
    // {
    //     // One level popped
    //     let segments = url.path_segments_mut();
    //     if let Ok(mut segments) = segments {
    //         segments.pop();
    //     }
    // }
    //
    //
    // resolved_url = url.to_string();
    // if is_valid_url(&resolved_url).await {
    //     return resolved_url;
    // }
    //
    //
    // href.to_string() // If no valid resolution found, return the original href
}

/// Checks if a URL is valid by performing a HEAD request.
///
/// # Arguments
///
/// * `url` - The URL to check.
///
/// # Returns
///
/// A boolean indicating whether the URL is valid or not.
async fn is_valid_url(url: &str) -> bool {
    let client = reqwest::Client::new();
    match client.head(url).send().await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}


/// Wgets file asynchronously
///
/// # Arguments
///
/// * `url` - The link to fetch
///
/// # Returns
///
/// Result<(), Box<dyn std::error::Error>>
pub(crate) async fn wget(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::process::Command;

    if !is_valid_url(url).await {
        return Err("Invalid URL".into());
    }

    // Initialize the Command with 'wget'
    let mut cmd = Command::new("wget");

    // Adding necessary arguments to wget
    cmd.arg("--no-check-certificate");
    cmd.arg("-erobots=off");

    // Use -r to recursively save files
    cmd.arg("-r");

    // Add the URL as the last argument
    cmd.arg(url);

    // Execute the command and await its completion
    let output = cmd.output().await?;

    // Check if the wget command was successful
    if !output.status.success() {
        println!("wget failed with status: {}", output.status);
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        return Err("wget failed".into());
    }

    Ok(())
}