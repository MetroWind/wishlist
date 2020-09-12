use reqwest;

use crate::error::Error;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:80.0) Gecko/20100101 Firefox/80.0";

pub async fn get(url: &str) -> Result<String, Error>
{
    let client = reqwest::Client::new();
    client.get(url).header("User-Agent", USER_AGENT)
        .send().await.map_err(
            |_| error!(RuntimeError, format!("Failed to get {}", url)))?
        .text().await.map_err(
            |_| error!(RuntimeError,
                       format!("Failed to retrieve content from {}", url)))
}

pub async fn queryRequest(req: reqwest::RequestBuilder) -> Result<String, Error>
{
    req.header("User-Agent", USER_AGENT)
        .send().await.map_err(
            |_| error!(RuntimeError, "Failed to query"))?
        .text().await.map_err(
            |_| error!(RuntimeError, "Failed to retrieve content"))
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
