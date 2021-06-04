use reqwest;
use chrono;

use crate::error::Error;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:80.0) Gecko/20100101 Firefox/80.0";

pub async fn get(url: &str) -> Result<String, Error>
{
    let client = reqwest::Client::new();
    client.get(url).header("User-Agent", USER_AGENT)
        .send().await.map_err(|_| rterr!("Failed to get {}", url))?
        .text().await.map_err(
            |_| rterr!("Failed to retrieve content from {}", url))
}

pub async fn queryRequest(req: reqwest::RequestBuilder) -> Result<String, Error>
{
    req.header("User-Agent", USER_AGENT)
        .send().await.map_err(|_| rterr!("Failed to query"))?
        .text().await.map_err(|_| rterr!("Failed to retrieve content"))
}

pub fn findSubStr<'a>(s: &'a str, begin: &'a str, end: &'a str)
                      -> Option<(&'a str, usize, usize)>
{
    let start = if let Some(idx) = s.find(begin)
    {
        idx + begin.len()
    }
    else
    {
        return None;
    };

    let stop = if let Some(idx) = &s[start..].find(end)
    {
        start + idx
    }
    else
    {
        return None;
    };

    Some((&s[start..stop], start, stop))
}

/// “39.99” -> 3999, “15” -> 1500
pub fn parsePrice(price_raw: &str) -> Result<i64, Error>
{
    if price_raw.find(".").is_none()
    {
        price_raw.parse::<i64>().map_err(
            |_| rterr!("Failed to parse price: {}", price_raw))
            .map(|x| x * 100)
    }
    else
    {
        price_raw.replace(".", "").parse().map_err(
            |_| rterr!("Failed to parse price: {}", price_raw))
    }
}

pub fn timestampToUtcTime(ts: i64) -> chrono::DateTime<chrono::Utc>
{
    chrono::DateTime::<chrono::Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp(ts, 0), chrono::Utc)
}
