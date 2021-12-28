use anyhow::Context;

pub struct SearchResult {
    pub image: url::Url,
    pub page: url::Url,
}

fn get_keywords(raw: &str) -> Vec<&str> {
    raw.split(char::is_whitespace)
        .filter(|w| !w.is_empty())
        .collect()
}

pub async fn search_image(request: &str) -> anyhow::Result<Vec<SearchResult>> {
    let keywords = get_keywords(request);
    if keywords.is_empty() {
        return Ok(Vec::default());
    }

    log::info!("Search request: '{}'", request);

    let request_uri = format!(
        "https://dilbert.com/search_results?terms={}",
        keywords.join("+")
    );

    let resp = reqwest::get(request_uri).await?;
    process_search_image_response(resp).await
}

async fn process_search_image_response(
    resp: reqwest::Response,
) -> anyhow::Result<Vec<SearchResult>> {
    let body = resp.text().await?;
    let dom = scraper::Html::parse_document(body.as_str());
    let comic_container_selector = scraper::Selector::parse(".img-comic-container").unwrap();
    let comic_link_selector = scraper::selector::Selector::parse(".img-comic-link").unwrap();
    let comic_selector = scraper::selector::Selector::parse(".img-comic").unwrap();
    let search_results = dom
        .select(&comic_container_selector)
        .filter_map(|e| {
            let page = e
                .select(&comic_link_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .ok_or(anyhow::Error::msg("no href attr"))
                .and_then(|e| url::Url::parse(e).context("Can't parse url"));
            let img = e
                .select(&comic_selector)
                .next()
                .and_then(|e| e.value().attr("src"))
                .ok_or(anyhow::Error::msg("no src attr"))
                .and_then(|e| url::Url::parse(e).context("Can't parse url"));

            if page.is_ok() && img.is_ok() {
                let result = SearchResult {
                    page: page.unwrap(),
                    image: img.unwrap(),
                };
                Some(result)
            } else {
                None
            }
        })
        .collect::<Vec<SearchResult>>();

    log::info!("Search found {} images.", search_results.len());

    Ok(search_results)
}
