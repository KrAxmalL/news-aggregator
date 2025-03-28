use candid::{CandidType, Nat};
use ic_cdk::api::management_canister::http_request::{
    CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext, http_request,
};

use ic_cdk_macros::{query, update};
use serde::{Deserialize, Serialize};

// Sources for getting the news
struct SourceConfig {
    pub title: &'static str,
    pub url: &'static str,
}

// Response with the news for the end users
#[derive(Serialize, CandidType)]
struct AggregatedNewsResponse {
    pub sources: Vec<NewsSource>,
    pub error_message: Option<String>,
}

#[derive(Serialize, CandidType)]
struct NewsSource {
    pub source_name: String,
    pub response_code: Option<Nat>,
    pub error_message: Option<String>,
    pub news: Vec<Article>,
}

#[derive(Serialize, CandidType)]
struct Article {
    pub title: String,
    pub description: String,
    pub link: String,
}

// Structures representing the structure of the RSS XML response from the news sources
#[derive(Debug, Deserialize)]
struct XmlNewsResponse {
    pub channel: Channel,
}

#[derive(Debug, Deserialize)]
struct Channel {
    pub title: String,
    pub item: Vec<Item>,
}

#[derive(Debug, Deserialize)]
struct Item {
    pub title: String,
    pub description: String,
    pub link: String,
}

static NEWS_SOURCES: &[SourceConfig; 3] = &[
    SourceConfig {
        title: "BBC News",
        url: "https://feeds.bbci.co.uk/news/rss.xml",
    },
    SourceConfig {
        title: "POLITICO",
        url: "https://rss.politico.com/politics-news.xml",
    },
    SourceConfig {
        title: "The Guardian",
        url: "https://www.theguardian.com/world/rss",
    },
];

static DEFAULT_NEWS_AMOUNT: usize = 3;

// 256 Kilobytes - used to reduce the cost of calling external services
static MAX_RESPONSE_SIZE_BYTES: u64 = 262144;

#[update]
async fn get_aggregated_news() -> AggregatedNewsResponse {
    fetch_aggregated_news(DEFAULT_NEWS_AMOUNT).await
}

#[update]
async fn get_aggregated_news_limited(news_per_source: u8) -> AggregatedNewsResponse {
    if news_per_source == 0 {
        AggregatedNewsResponse {
            sources: vec![],
            error_message: Some("news_per_source must be > 0".to_string()),
        }
    } else {
        fetch_aggregated_news(usize::from(news_per_source)).await
    }
}

async fn fetch_aggregated_news(news_per_source: usize) -> AggregatedNewsResponse {
    let news_sources = futures::future::join_all(
        NEWS_SOURCES
            .into_iter()
            .map(|source| get_news_source(source, news_per_source)),
    )
    .await
    .into_iter()
    .collect();

    AggregatedNewsResponse {
        sources: news_sources,
        error_message: None,
    }
}

fn build_news_request(url: &str) -> CanisterHttpRequestArgument {
    let request_headers = vec![HttpHeader {
        name: "User-Agent".to_string(),
        value: "news-aggregator-backend-canister".to_string(),
    }];
    CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        headers: request_headers,
        body: None,
        max_response_bytes: Some(MAX_RESPONSE_SIZE_BYTES),
        transform: Some(TransformContext::new(transform_response, Vec::new())),
    }
}

async fn get_news_source(source_config: &SourceConfig, news_per_source: usize) -> NewsSource {
    let request: CanisterHttpRequestArgument = build_news_request(source_config.url);
    match http_request(request).await {
        Ok((response,)) => parse_xml_http_response(source_config, news_per_source, response),
        Err((code, message)) => NewsSource {
            source_name: source_config.title.to_string(),
            response_code: None,
            error_message: Some(format!(
                "http_request finished with error. RejectionCode={code:?}, Error={message}"
            )),
            news: vec![],
        },
    }
}

fn parse_xml_http_response(
    source_config: &SourceConfig,
    news_per_source: usize,
    http_response: HttpResponse,
) -> NewsSource {
    match String::from_utf8(http_response.body) {
        Ok(body_as_string) => {
            match quick_xml::de::from_str::<XmlNewsResponse>(body_as_string.as_str()) {
                Ok(xml_news_response) => NewsSource {
                    source_name: source_config.title.to_string(),
                    response_code: Some(http_response.status),
                    error_message: None,
                    news: xml_news_response
                        .channel
                        .item
                        .iter()
                        .map(|i| Article {
                            title: i.title.clone(),
                            description: i.description.clone(),
                            link: i.link.clone(),
                        })
                        .take(news_per_source)
                        .collect(),
                },
                Err(error) => NewsSource {
                    source_name: source_config.title.to_string(),
                    response_code: Some(http_response.status),
                    error_message: Some(format!(
                        "Failed to parse XML response body. Error={error:?}"
                    )),
                    news: vec![],
                },
            }
        }
        Err(error) => NewsSource {
            source_name: source_config.title.to_string(),
            response_code: Some(http_response.status),
            error_message: Some(format!("Failed to decode response body. Error={error:?}")),
            news: vec![],
        },
    }
}

// Remove headers from the response, they are not used
#[query]
fn transform_response(raw: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        ..Default::default()
    }
}
