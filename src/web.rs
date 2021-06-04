use serde::{Deserialize, Serialize};
use tokio;
use warp;
use warp::reject::Reject;
use warp::{Filter, Rejection};
use warp::reply::Reply;
use log::error as log_error;
use log::info;

use crate::error::Error as Error;
use crate::data;
use crate::config;
use crate::store;

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

impl Reject for Error {}

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

#[derive(Deserialize, Serialize)]
struct PricePoint
{
    time: u64,
    price: f64,
}

#[derive(Clone)]
pub struct WebHandler
{
    db_file: String,
    port: u16,
    url_prefix: Option<String>,
}

/// Take the 1st Result as argument. If Result is an error, let the
/// current function return a 500 error, otherwide run the 2nd
/// argument on the unwrapped value of the 1st argument. The 2nd
/// argument is an expression that do some thing with the unwrapped
/// value. The unwrapped value is assumed to be called “ok”.
macro_rules! web_error
{
    ($stuff:expr) =>
    {
        $stuff.map_err(|e| warp::reject::custom(e))?
    }
}

impl WebHandler
{
    pub fn new(conf: &config::ConfigParams) -> Self
    {
        Self {
            db_file: conf.db_file.clone(),
            port: conf.port,
            url_prefix: conf.url_prefix.clone(),
        }
    }

    async fn list(self) -> Result<Box<dyn Reply>, Rejection>
    {
        let mut d = data::DataManager::new(data::SqliteFilename::File(
            std::path::PathBuf::from(self.db_file)));
        web_error!(d.connect());
        let items: Vec<store::ItemInfo> = web_error!(d.getItems())
            .iter().map(|p| p.clone()).collect();
        Ok(Box::new(warp::reply::json(&items)))
    }

    async fn priceHistory(self, store: String, id: String) ->
        Result<Box<dyn Reply>, Rejection>
    {
        let mut d = data::DataManager::new(data::SqliteFilename::File(
            std::path::PathBuf::from(self.db_file)));
        web_error!(d.connect());
        let item = data::ItemKey{ store: store, id: id };
        let points: Vec<PricePoint> = d.priceHistory(item)?.iter().map(
            |p| PricePoint{
                time: p.time.timestamp() as u64,
                price: (p.price as f64) / 100.0,
            }).collect();

        Ok(Box::new(warp::reply::json(&points)))
    }

    pub fn start(self)
    {
        let port = self.port;
        let url_prefix: Option<String> = self.url_prefix.clone();
        let handler = self.clone();
        let route_list = warp::path(ENTRY).and(warp::path("list"))
            .and(warp::path::end())
            .and_then(move || { handler.clone().list() });
        let route_history = warp::path(ENTRY).and(warp::path("price_history"))
            .and(warp::path::param()).and(warp::path::param())
            .and_then(move |store: String, id: String| {
                self.clone().priceHistory(store, id)
            });

        let route_fe = warp::any().and(warp::fs::dir("frontend"));

        let rt = tokio::runtime::Runtime::new().unwrap();
        info!("Running service at http://127.0.0.1:{}/{}", port,
              url_prefix.as_ref().unwrap_or(&String::new()));
        rt.block_on(
            warp::serve(withOptionalPrefix(url_prefix)
                        .and(route_list.or(route_history).or(route_fe)))
                .try_bind(([127, 0, 0, 1], port)));
    }
}
