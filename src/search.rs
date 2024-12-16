// src/search.rs

use async_stream::stream;
use futures::Stream;
use reqwest::{Client};
use scraper::{Html, Selector};
use tokio::sync::mpsc::UnboundedSender;
use urlencoding::encode;
use regex::Regex;

/// Performs a site-specific internet search and sends found URLs to the crawler via the provided sender.
///
/// # Arguments
///
/// * `search_site` - The site to search (e.g., "example.com").
/// * `search_limit` - Maximum number of search results to retrieve.
/// * `url_sender` - Sender to send found URLs to the crawler.
pub async fn search_site_urls(
    search_site: &str,
    search_limit: u32,
    url_sender: UnboundedSender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Build the HTTP client with a user agent
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)")
        .build()?;

    let mut results_fetched = 0;
    let mut start_index = 0;

    while results_fetched < search_limit {
        let search_query = format!("site:{}", search_site);
        let url = format!(
            "https://www.google.com/search?q={}&start={}",
            encode(&search_query),
            start_index
        );

        // Fetch the search results page
        let resp = client.get(&url).send().await?;
        let body = resp.text().await?;

        // Parse the HTML to extract result links
        let document = Html::parse_document(&body);
        let selector = Selector::parse("a").unwrap();
        let links = document.select(&selector);
        let links: Vec<_> = links
            .map(|link| link.value().attr("href"))
            .filter_map(|opt| opt)
            .collect();
        let pattern = Regex::new(r"/url\?q=(.*?)&sa=").unwrap();
        let mut found_urls = Vec::new();

        for link in links {
            if let Some(caps) = pattern.captures(&link) {
                if let Some(url_match) = caps.get(1) {
                    let url = url_match.as_str();
                    found_urls.push(url.to_string());
                }
            }
        }

        if found_urls.is_empty() {
            // No more results
            break;
        }

        for link in found_urls {
            if results_fetched >= search_limit {
                break;
            }
            url_sender
                .send(link.clone())
                .unwrap_or_else(|err| eprintln!("Error sending URL from search: {}", err));
            results_fetched += 1;
        }

        start_index += 10; // Assuming each page has 10 results
    }

    Ok(())
}

/// Performs a site-specific internet search and returns a stream of found URLs.
///
/// # Arguments
///
/// * `search_site` - The site to search (e.g., "example.com").
/// * `search_limit` - Maximum number of search results to retrieve.
pub fn search_site_urls_stream(
    search_site: &str,
    search_limit: u32,
) -> impl Stream<Item = String> {
    let search_site = search_site.to_string();
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)")
        .build()
        .expect("Failed to build HTTP client");

    stream! {
        let mut results_fetched = 0;
        let mut start_index = 0;

        while results_fetched < search_limit {
            let search_query = format!("site:{}", search_site);
            let url = format!(
                "https://www.google.com/search?q={}&start={}",
                encode(&search_query),
                start_index
            );

            // Fetch the search results page
            let resp = client.get(&url).send().await;
            if let Ok(resp) = resp {
                if let Ok(body) = resp.text().await {
                    // Parse the HTML to extract result links
                    let document = Html::parse_document(&body);
                    let selector = Selector::parse("a").unwrap();
                    let links = document.select(&selector);
                    let links: Vec<_> = links
                        .map(|link| link.value().attr("href"))
                        .filter_map(|opt| opt)
                        .collect();
                    let pattern = Regex::new(r"/url\?q=(.*?)&sa=").unwrap();

                    for link in links {
                        if let Some(caps) = pattern.captures(&link) {
                            if let Some(url_match) = caps.get(1) {
                                let url = url_match.as_str();
                                yield url.to_string();
                                results_fetched += 1;
                                if results_fetched >= search_limit {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if results_fetched >= search_limit {
                break;
            }
            start_index += 10; // Assuming each page has 10 results
        }
    }
}