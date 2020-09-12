use serde::{Deserialize, Serialize};
use tokio;
use warp::{Filter, Rejection};
use warp::reply::Reply;
use warp::http;
use log::error as log_error;

use crate::store;
use crate::data;

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

async fn get(q: data::ItemKey) -> Result<Box<dyn Reply>, Rejection>
{
    let s = match store::Store::new(&q.store)
    {
        Ok(s) => s,
        Err(_) => { return Ok(replyError(500, "Invalid store")); },
    };

    match s.get(&q.id).await
    {
        Ok(i) => Ok(Box::new(warp::reply::json(&i))),
        Err(_) => Ok(replyError(500, "Failed to get price")),
    }
}

pub fn start(prefix: Option<String>)
{
    let route_list = warp::path(ENTRY).and(warp::path("list"))
        .and(warp::path::end())
        .and_then(list);

    let route_get = warp::path(ENTRY).and(warp::path("get"))
        .and(warp::query::<data::ItemKey>())
        .and_then(get);

    let route_fe = warp::any().and(warp::fs::dir("frontend"));

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(warp::serve(withOptionalPrefix(prefix)
                            .and(route_list.or(route_get).or(route_fe)))
                .run(([127, 0, 0, 1], 8000)));
}
