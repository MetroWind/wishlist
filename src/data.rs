use rusqlite as sql;
use rusqlite::OptionalExtension;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error;
use crate::error::Error as Error;
use crate::store::ItemInfo;

pub enum SqliteFilename { InMemory, File(std::path::PathBuf) }

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct ItemKey
{
    pub store: String,
    pub id: String,
}

impl ItemKey
{
    pub fn fromItem(item: &ItemInfo) -> Self
    {
        Self { store: item.store.clone(), id: item.id.clone() }
    }
}

pub struct DataManager
{
    filename: SqliteFilename,
    connection: Option<sql::Connection>,
}

impl DataManager
{
    pub fn new(f: SqliteFilename) -> Self
    {
        Self { filename: f, connection: None }
    }

    pub fn newWithFilename(f: &str) -> Self
    {
        Self {
            filename: SqliteFilename::File(std::path::PathBuf::from(f)),
            connection: None
        }
    }

    fn confirmConnection(&self) -> Result<&sql::Connection, Error>
    {
        if let Some(c) = &self.connection
        {
            Ok(c)
        }
        else
        {
            Err(error!(DataError, "Sqlite database not connected"))
        }
    }

    /// Connect to the database. Create database file if not exist.
    pub fn connect(&mut self) -> Result<(), Error>
    {
        let conn = match &self.filename
        {
            SqliteFilename::File(path) => sql::Connection::open(path).map_err(
                |e| error!(DataError, "Failed to create database: {}", e))?,
            SqliteFilename::InMemory =>
                sql::Connection::open_in_memory().map_err(
                    |e| error!(DataError, "Failed to create database: {}", e))?,
        };
        self.connection = Some(conn);
        Ok(())
    }

    pub fn init(&self) -> Result<(), Error>
    {
        let conn = self.confirmConnection()?;
        conn.execute(
            "CREATE TABLE wishlist (
                  internal_id INTEGER PRIMARY KEY ASC,
                  store TEXT,
                  id TEXT,
                  name TEXT,
                  url TEXT
                  )", []).map_err(
            |e| error!(DataError, "Failed to create table: {}", e))?;

        conn.execute(
            "CREATE TABLE price (
                  time INTEGER,
                  price INTEGER, -- price * 100
                  price_str TEXT,
                  store TEXT,
                  id TEXT,
                  item_id INTEGER,
                  FOREIGN KEY(item_id) REFERENCES wishlist(internal_id)
                  )", []).map_err(
            |e| error!(DataError, "Failed to create table: {}", e))?;
        Ok(())
    }

    pub fn addItem(&self, item: &ItemInfo) -> Result<(), Error>
    {
        if self.findItem(ItemKey::fromItem(item))?.is_some()
        {
            return Ok(());
        }
        let conn = self.confirmConnection()?;
        let changed_row_count = conn.execute(
            "INSERT INTO wishlist (store, id, name, url)
             VALUES (?, ?, ?, ?)",
            [&item.store, &item.id, &item.name, &item.url]).map_err(
            |_| error!(DataError, "Failed to add item"))?;
        if changed_row_count != 1
        {
            return Err(error!(DataError, "Invalid insert happened"));
        }
        Ok(())
    }

    fn rowToItem(row: &sql::Row) -> sql::Result<(i64, ItemInfo)>
    {
        let store: String = row.get(1)?;
        let id: String = row.get(2)?;
        let mut result = ItemInfo::new(&store, &id);
        result.name = row.get(3)?;
        result.url = row.get(4)?;
        Ok((row.get(0)?, result))
    }

    pub fn getItems(&self) -> Result<Vec<ItemInfo>, Error>
    {
        let conn = self.confirmConnection()?;
        let mut cmd = conn.prepare("SELECT * FROM wishlist;").map_err(
            |_| error!(DataError,
                       "Failed to compile statement to get all items"))?;
        let iter = cmd.query_map([], Self::rowToItem).map_err(
            |e| error!(DataError, "Failed to get all items: {}", e))?;
        let mut result: Vec<ItemInfo> = Vec::new();
        for item_pair in iter
        {
            let item_pair = item_pair.map_err(
                |_| error!(DataError, "Failed to get one of the items"))?;
            let mut cmd = conn.prepare("SELECT price, price_str FROM price WHERE
                item_id = ? ORDER BY time DESC LIMIT 1").map_err(
                |_| error!(DataError,
                           "Failed to compile statement to get price"))?;
            // Item_pair is (rowid, item).
            result.push(
                cmd.query_row([item_pair.0], |row| {
                    let mut item = item_pair.1;
                    item.price = row.get(0)?;
                    item.price_str = row.get(1)?;
                    Ok(item)
                }).map_err(|_| error!(DataError, "Failed to get price"))?);
        }
        Ok(result)
    }

    fn findItem(&self, key: ItemKey) -> Result<Option<i64>, Error>
    {
        let conn = self.confirmConnection()?;
        conn.query_row(
            "SELECT internal_id FROM wishlist WHERE store = ? AND id = ?",
            [&key.store, &key.id],
            |row| row.get(0)).optional().map_err(
            |e| error!(DataError, "Failed to find item: {}", e))
    }

    pub fn addPrice(&self, item: &ItemInfo) -> Result<(), Error>
    {
        let row_id = self.findItem(ItemKey::fromItem(item))?.ok_or_else(
            || rterr!("Unknown item"))?;
        let now = Utc::now();
        let conn = self.confirmConnection()?;
        conn.execute("INSERT INTO price (time, price, price_str, store, id,
                                         item_id)
                      VALUES (?, ?, ?, ?, ?, ?)",
                     sql::params![now.timestamp(), item.price, item.price_str,
                                  item.store, item.id, row_id]).map_err(
            |e| error!(DataError, "Failed to add price: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests
{
    use std::thread::sleep;
    use std::time;
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    type AnyError = Box<dyn std::error::Error>;

    fn newDataManager() -> Result<DataManager, Error>
    {
        let mut data = DataManager::new(SqliteFilename::InMemory);
        data.connect()?;
        data.init()?;
        Ok(data)
    }

    #[test]
    fn db_creation() -> Result<(), AnyError>
    {
        let mut data = newDataManager()?;
        let conn = data.confirmConnection()?;
        let mut cmd = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND
             name NOT LIKE 'sqlite_%' ORDER BY 1")?;
        let iter = cmd.query_map([], |row| row.get(0))?;
        let mut tables: Vec<String> = Vec::new();
        for i in iter
        {
            let i = i?;
            tables.push(i);
        }
        assert_eq!(tables.len(), 2);
        assert_eq!(tables[0], "price");
        assert_eq!(tables[1], "wishlist");
        Ok(())
    }

    #[test]
    fn add_find_item() -> Result<(), AnyError>
    {
        let data = newDataManager()?;
        let mut item = ItemInfo::new("store", "id");
        item.name = "aaa".to_owned();
        item.url = "bbb".to_owned();
        data.addItem(&item)?;
        assert!(data.findItem(ItemKey::fromItem(&item))?.is_some());
        Ok(())
    }

    #[test]
    fn add_price_get_items_basic() -> Result<(), AnyError>
    {
        let data = newDataManager()?;
        let mut item1 = ItemInfo::new("switch", "id1");
        item1.name = "aaa".to_owned();
        item1.url = "bbb".to_owned();
        item1.price = 100;
        item1.price_str = "$1.00".to_owned();

        data.addItem(&item1)?;
        data.addPrice(&item1)?;
        let items = data.getItems()?;
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, item1.name);
        Ok(())
    }

    #[test]
    fn add_price_get_items_multiple() -> Result<(), AnyError>
    {
        let data = newDataManager()?;

        let mut item1 = ItemInfo::new("switch", "id1");
        item1.name = "aaa".to_owned();
        item1.url = "bbb".to_owned();
        item1.price = 100;
        item1.price_str = "$1.00".to_owned();

        let mut item2 = ItemInfo::new("ps", "id2");
        item2.name = "xxx".to_owned();
        item2.url = "yyy".to_owned();
        item2.price = 200;
        item2.price_str = "$2.00".to_owned();

        data.addItem(&item1)?;
        data.addPrice(&item1)?;
        data.addItem(&item2)?;
        data.addPrice(&item2)?;
        item2.price = 300;
        item2.price_str = "$3.00".to_owned();
        sleep(time::Duration::new(2, 0));
        data.addPrice(&item2)?;

        let items = data.getItems()?;
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, item1.name);
        assert_eq!(items[0].url, item1.url);
        assert_eq!(items[0].price, item1.price);
        assert_eq!(items[0].price_str, item1.price_str);
        assert_eq!(items[1].name, item2.name);
        assert_eq!(items[1].url, item2.url);
        assert_eq!(items[1].price, item2.price);
        assert_eq!(items[1].price_str, item2.price_str);

        let conn = data.confirmConnection()?;
        let count_item2_prices: i64 = conn.query_row(
            "SELECT COUNT(*) FROM price where id = ?",
            [item2.id], |row| row.get(0))?;
        assert_eq!(count_item2_prices, 2);

        Ok(())
    }
}
