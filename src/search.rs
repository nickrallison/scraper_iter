use reqwest::{Client, Url};
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
        let links: Vec<_> = links.map(|link| link.attr("href")).filter(|opt| opt.is_some()).map(|opt| opt.unwrap()).collect();
        let pattern = Regex::new(r"\/url\?q=(.*?)&sa=").unwrap();
        let mut found_urls = Vec::new();
        for link in links {
            let url = pattern.captures(&link).and_then(|caps| caps.get(1).map(|m| m.as_str()));
            if let Some(url) = url {
                found_urls.push(url.to_string());
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