use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::collections::HashMap;
use std::time;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json as json;
use log::{info,debug};

use crate::error::Error;
use crate::store;

const DATA_FILE: &str = "wishlist.json";

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct ItemKey
{
    pub store: String,
    pub id: String,
}

pub struct CachedItem
{
    pub item: store::ItemInfo,
    time: time::SystemTime,
}

impl CachedItem
{
    /// Info that is older than this needs a refresh.
    fn maxAge() -> time::Duration
    {
        time::Duration::new(3600, 0)
    }

    pub fn new(item: store::ItemInfo, t: time::SystemTime) -> Self
    {
        Self{ item: item, time: t }
    }

    pub fn expired(&self) -> bool
    {
        time::SystemTime::now().duration_since(self.time).unwrap()
            > Self::maxAge()
    }
}

pub struct Cache
{
    cache: Mutex<HashMap<ItemKey, CachedItem>>,
}

impl Cache
{
    pub fn new() -> Self
    {
        Self{ cache: Mutex::new(HashMap::new()) }
    }

    fn getCachedUnsafe(&self, key: &ItemKey) -> store::ItemInfo
    {
        let cache = self.cache.lock().unwrap();
        cache.get(key).unwrap().item.clone()
    }

    pub async fn get(&self, key: &ItemKey) -> Result<store::ItemInfo, Error>
    {
        {
            let cache = self.cache.lock().unwrap();
            if let Some(i) = cache.get(key)
            {
                debug!("Reading cached item...");
                if !i.expired()
                {
                    return Ok(i.item.clone());
                }
            }
        }

        debug!("Cached item expired or nonexists. Refreshing...");
        let s = store::Store::new(&key.store)?;
        let now = time::SystemTime::now();
        let item = s.get(&key.id).await?;
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key.clone(), CachedItem::new(item, now));
        }
        Ok(self.getCachedUnsafe(key))
    }
}

pub fn getItems() -> Result<Vec<ItemKey>, Error>
{
    let mut file = File::open(DATA_FILE).map_err(
        |_| error!(RuntimeError, "Failed to open data file"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(
        |_| error!(RuntimeError, "Failed to read data file"))?;

    let items: Vec<ItemKey> = json::from_str(&contents).map_err(
        |_| error!(RuntimeError, "Failed to parse JSON"))?;
    Ok(items)
}

fn saveItems(items: Vec<ItemKey>) -> Result<(), Error>
{
    let mut file = File::create(DATA_FILE).map_err(
        |_| error!(RuntimeError, "Failed to open/create data file"))?;

    file.write_all(&json::to_vec_pretty(&items).map_err(
        |_| error!(RuntimeError, "Failed to serialize data"))?).map_err(
        |_| error!(RuntimeError, "Failed to write dwata file"))
}

pub fn addItem(item: &ItemKey) -> Result<(), Error>
{
    let mut items = getItems()?;
    items.push(item.clone());
    saveItems(items)
}

pub fn removeItem(item: &ItemKey) -> Result<(), Error>
{
    let mut items = getItems()?;
    for i in 0..items.len()
    {
        if &items[i] == item
        {
            items.remove(i);
            break;
        }
    }
    saveItems(items)
}

pub fn initialize() -> Result<(), Error>
{
    let path = Path::new(DATA_FILE);
    if !path.exists()
    {
        info!("Creating data store...");
        saveItems(Vec::new())?;
    }
    Ok(())
}
