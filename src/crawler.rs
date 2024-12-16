use async_stream::stream;
use futures::stream::FuturesUnordered;
use futures::Stream;
use futures::StreamExt;
use reqwest;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;

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
pub fn crawl_urls<F>(initial_urls: Vec<String>, filter: F) -> impl Stream<Item = String>
where
    F: Fn(&String) -> bool + Send + Sync + 'static,
{
    let filter = Arc::new(filter);
    Box::pin(stream! {
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
    })
}

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

fn resolve_url(href: &str, base_url: &str) -> String {
    match reqwest::Url::parse(base_url) {
        Ok(base) => match base.join(href) {
            Ok(url) => url.to_string(),
            Err(_) => href.to_string(),
        },
        Err(_) => href.to_string(),
    }
}