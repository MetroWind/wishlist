use std::path;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use log::info;
use log::error as log_error;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use std::process::Stdio;

use crate::error::Error;
use crate::config;
use crate::store;
use crate::data;

pub fn maybeInitDB(conf: &config::ConfigParams) -> Result<(), Error>
{
    let db_path = path::Path::new(&conf.db_file);
    if !db_path.exists()
    {
        info!("Database not found. Creating new database at {}...",
              conf.db_file);
        let mut d = data::DataManager::new(data::SqliteFilename::File(
            PathBuf::from(db_path)));
        d.connect()?;
        d.init()
    }
    else
    {
        Ok(())
    }
}

async fn updatePrice(item: store::ItemInfo) -> Result<store::ItemInfo, Error>
{
    let s = store::Store::new(&item.store)?;
    s.get(&item.id).await
}

async fn alert(item: &store::ItemInfo) -> Result<(), Error>
{
    let msg = format!("Price of [{}]({}) is now at {}!",
                      item.name, item.url, item.price_str);
    let mut child = Command::new("telegram-notify-bot")
        .stdin(Stdio::piped()).spawn().map_err(
            |_| rterr!("Failed to spawn telegram-notify-bot"))?;
    let child_stdin = child.stdin.as_mut().unwrap();
    child_stdin.write_all((&msg).as_bytes()).await.map_err(
        |_| rterr!("Failed to write message to telegram-notify-bot"))?;
    child.wait().await.map_err(|_| rterr!("telegram-notify-bot failed"))?;
    Ok(())
}

async fn updatePrices(items: Vec<store::ItemInfo>, default_interval: Duration) ->
    Result<Vec<store::ItemInfo>, Error>
{
    let now = Utc::now();
    let mut result: Vec<store::ItemInfo> = Vec::new();
    for item in items
    {
        let interval = chrono::Duration::from_std(
            item.update_interval.or(Some(default_interval)).unwrap()).map_err(
            |_| rterr!("Failed to convert interval"))?;
        // Allow 1 minute fluctuation
        if item.last_update + interval < now + chrono::Duration::minutes(1)
        {
            let orig_price = item.price;
            let name = item.name.clone();
            let item_with_new_price = match updatePrice(item).await
            {
                Ok(p) => p,
                Err(e) =>
                {
                    log_error!("Failed to update price for {}: {}", name, e);
                    continue;
                }
            };
            if item_with_new_price.price == orig_price
            {
                continue;
            }
            info!("Price of {} changed to {}.",
                  item_with_new_price.name, item_with_new_price.price_str);
            if item_with_new_price.price < orig_price
            {
                if let Err(err) = alert(&item_with_new_price).await
                {
                    log_error!("{}", err);
                }
            }
            result.push(item_with_new_price);
        }
    }
    Ok(result)
}

pub fn addItem(store: &str, id: &str, conf: config::ConfigParams) ->
    Result<(), Error>
{
    maybeInitDB(&conf)?;
    // Try to get price
    let rt = tokio::runtime::Runtime::new().map_err(
        |_| rterr!("Failed to create runtime"))?;
    let key = data::ItemKey{store: store.to_owned(), id: id.to_owned()};
    let item = rt.block_on(store::Store::new(&key.store)?.get(&key.id))?;
    info!("Adding {} at {}...", item.name, item.price_str);
    let mut d = data::DataManager::newWithFilename(&conf.db_file);
    d.connect()?;
    d.addItem(&item)?;
    d.addPrice(&item)
}

pub fn removeItem(store: &str, id: &str, conf: config::ConfigParams) ->
    Result<(), Error>
{
    let key = data::ItemKey{store: store.to_owned(), id: id.to_owned()};
    maybeInitDB(&conf)?;
    let mut d = data::DataManager::newWithFilename(&conf.db_file);
    d.connect()?;
    d.removeItem(key)
}

pub fn listItems(conf: config::ConfigParams) -> Result<(), Error>
{
    let mut d = data::DataManager::newWithFilename(&conf.db_file);
    d.connect()?;
    let items = d.getItems()?;
    for item in items
    {
        println!("{} {} {} {}", item.store, item.id, item.name,
                 item.price_str);
    }
    Ok(())
}

pub fn updateItemPrices(conf: config::ConfigParams) -> Result<(), Error>
{
    let default_interval = Duration::new(conf.update_interval_sec, 0);

    let mut d = data::DataManager::newWithFilename(&conf.db_file);
    d.connect()?;
    let rt = tokio::runtime::Runtime::new().map_err(
        |_| rterr!("Failed to create runtime"))?;
    info!("Updating prices...");
    let items = rt.block_on(
        updatePrices(d.getItems()?, default_interval))?;
    for item in items
    {
        d.addPrice(&item)?;
    }
    Ok(())
}
