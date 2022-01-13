use anyhow::Context;

use crate::dilbert::cache::DILBERT_CACHE;
use crate::dilbert::tags::{ParseTags, Tag};

#[derive(Clone)]
pub struct SearchResult {
    pub image: url::Url,
    pub page: url::Url,
}

pub struct SearchSettings {
    use_cache: bool,
}

impl SearchSettings {
    pub(crate) fn from_env() -> SearchSettings {
        SearchSettings {
            use_cache: match std::env::var("DILBERT_BOT_USE_CACHE") {
                Ok(v) => v.parse::<bool>().unwrap_or_default(),
                Err(_) => false,
            },
        }
    }
}

pub async fn search_image(
    request: &str,
    settings: &SearchSettings,
) -> anyhow::Result<Vec<SearchResult>> {
    log::info!("Search request: '{}'", request);

    let tags = request.parse_tags()?;

    if settings.use_cache {
        cached_search(&tags).await
    } else {
        non_cached_search(&tags).await
    }
}

async fn cached_search(tags: &[Tag]) -> anyhow::Result<Vec<SearchResult>> {
    match search_image_in_cache(tags) {
        ok @ Ok(_) => ok,
        Err(_) => {
            let results = search_image_on_web(tags).await?;
            cache_results(&results);
            Ok(results.into_iter().map(|res| res.search_result).collect())
        }
    }
}

async fn non_cached_search(tags: &[Tag]) -> anyhow::Result<Vec<SearchResult>> {
    Ok(search_image_on_web(tags)
        .await?
        .into_iter()
        .map(|r| r.search_result)
        .collect())
}

fn search_image_in_cache(tags: &[Tag]) -> anyhow::Result<Vec<SearchResult>> {
    log::info!("Try find in cache: '{:?}'", tags);
    let search_limit = 10;
    let result_from_cache = DILBERT_CACHE.find(&tags, search_limit);
    log::info!("Found in cache: '{}'", result_from_cache.len());

    if result_from_cache.len() < search_limit / 2 {
        Err(anyhow::anyhow!("Too few results."))
    } else {
        Ok(result_from_cache)
    }
}

fn cache_results(results: &[FullSearchResult]) {
    for res in results {
        DILBERT_CACHE.add(&res.tags, &res.search_result);
    }
}

struct FullSearchResult {
    search_result: SearchResult,
    tags: Vec<Tag>,
}

async fn search_image_on_web(tags: &[Tag]) -> anyhow::Result<Vec<FullSearchResult>> {
    log::info!("Search on web: '{:?}'", tags);

    let request_uri = format!(
        "https://dilbert.com/search_results?terms={}",
        tags.iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join("+")
    );

    let resp = reqwest::get(request_uri).await?;
    process_search_image_response(resp).await
}

async fn process_search_image_response(
    resp: reqwest::Response,
) -> anyhow::Result<Vec<FullSearchResult>> {
    let body = resp.text().await?;
    let dom = scraper::Html::parse_document(body.as_str());
    let comic_container_selector = scraper::Selector::parse(".img-comic-container").unwrap();
    let comic_link_selector = scraper::selector::Selector::parse(".img-comic-link").unwrap();
    let comic_selector = scraper::selector::Selector::parse(".img-comic").unwrap();
    let comic_tags_selector = scraper::selector::Selector::parse(".comic-tags > a").unwrap();
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
            let tags = e
                .select(&comic_tags_selector)
                .map(|e| e.inner_html().as_str().parse_tags())
                .flat_map(|maybe_tags| maybe_tags.unwrap_or_default())
                .collect::<Vec<Tag>>();

            if page.is_ok() && img.is_ok() && !tags.is_empty() {
                let search_result = SearchResult {
                    page: page.unwrap(),
                    image: img.unwrap(),
                };
                let result = FullSearchResult {
                    search_result,
                    tags,
                };
                Some(result)
            } else {
                None
            }
        })
        .collect::<Vec<FullSearchResult>>();

    log::info!("Search found {} images.", search_results.len());

    Ok(search_results)
}
