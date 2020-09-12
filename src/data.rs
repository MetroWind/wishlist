use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json as json;
use log::info;

use crate::error::Error;

const DATA_FILE: &str = "wishlist.json";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub struct ItemKey
{
    pub store: String,
    pub id: String,
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
