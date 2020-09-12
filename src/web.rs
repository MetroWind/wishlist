use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio;
use warp::{Filter, Rejection};
use warp::reply::Reply;
use warp::http;
use log::error as log_error;
use log::info;

use crate::data;
use crate::config;

const ENTRY: &str = "api";

#[derive(Deserialize, Serialize)]
struct ResError
{
    msg: String,
}

impl From<&str> for ResError
{
    fn from(s: &str) -> Self
    {
        Self { msg: s.to_owned() }
    }
}

fn withOptionalPrefix(prefix: Option<String>) ->
    warp::filters::BoxedFilter<()>
{
    match prefix
    {
        Some(s) => {
            let mut segs: Vec<&str> = s.split('/').collect();
            if segs.len() > 2
            {
                segs.truncate(2);
                log_error!("Service prefix too long, using {}", segs.join("/"));
            }
            if segs.len() == 1
            {
                warp::get().and(warp::path(s)).boxed()
            }
            else                // len = 2
            {
                warp::get().and(warp::path(segs[0].to_owned()))
                    .and(warp::path(segs[1].to_owned()))
                    .boxed()
            }
        },
        None => warp::get().boxed(),
    }
}

fn replyError(code: u16, msg: &str) -> Box<dyn Reply>
{
    Box::new(warp::reply::with_status(warp::reply::json(&ResError::from(msg)),
                                      http::StatusCode::from_u16(code).unwrap()))
}

async fn list() -> Result<Box<dyn Reply>, Rejection>
{
    let items = match data::getItems()
    {
        Ok(is) => is,
        Err(e) => { return Ok(replyError(500, &e.to_string())); },
    };
    Ok(Box::new(warp::reply::json(&items)))
}

async fn get(q: data::ItemKey, cache: &data::Cache) -> Result<Box<dyn Reply>, Rejection>
{
    match cache.get(&q).await
    {
        Ok(i) => Ok(Box::new(warp::reply::json(&i))),
        Err(e) => Ok(replyError(500, &e.to_string())),
    }
}

pub fn start(conf: &config::ConfigParams)
{
    let cache = Arc::new(data::Cache::new());
    let route_list = warp::path(ENTRY).and(warp::path("list"))
        .and(warp::path::end())
        .and_then(list);

    let route_get = warp::path(ENTRY).and(warp::path("get"))
        .and(warp::query::<data::ItemKey>())
        .and_then(move |key: data::ItemKey| {
            let cache = cache.clone();
            async move { get(key, &cache).await }
        });

    let route_fe = warp::any().and(warp::fs::dir("frontend"));

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    info!("Running service at http://127.0.0.1:{}/{}", conf.port,
          conf.url_prefix.as_ref().unwrap_or(&String::new()));
    rt.block_on(warp::serve(withOptionalPrefix(conf.url_prefix.clone())
                            .and(route_list.or(route_get).or(route_fe)))
                .try_bind(([127, 0, 0, 1], conf.port)));
}
