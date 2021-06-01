use serde_json as json;
use reqwest;

use crate::error::Error;
use crate::store::ItemInfo;
use crate::utils;

pub enum Region
{
    US,
}

pub struct Switch
{
    country: &'static str,
    lang: &'static str,
    name: &'static str,
}

impl Switch
{
    pub fn new(region: Region) -> Self
    {
        match region
        {
            Region::US => Self{ country: "US", lang: "en", name: "switch-us" },
        }
    }

    fn dataURL(&self, id: &str) -> String
    {
        format!("https://www.nintendo.com/games/detail/{}/", id)
    }

    fn getTitleID(content: &str) -> Result<(&str, &str), Error>
    {
        // Get position of ‘{’.
        let (store_data_str, _, _) = utils::findSubStr(
            content, "Object.freeze({", "});").ok_or(
            rterr!("Failed to find store data"))?;

        let (title, _, _) = utils::findSubStr(
            store_data_str, "title: \"", "\",")
            .ok_or(rterr!("Failed to find store data"))?;

        // let title: String = serde_json::from_str(
        //     &store_data_str[title_json_begin..title_json_end+1]).map_err(
        //     |_| rterr!("Failed to parse title JSON"))?;

        let (real_id, _, _) = utils::findSubStr(store_data_str, "nsuid: \"", "\",")
            .ok_or(rterr!( "Failed to find real ID"))?;
        Ok((title, real_id))
    }

    pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
    {
        // Get title and real ID.
        let store_url = self.dataURL(id);
        let content = utils::get(&store_url).await?;

        let (title, real_id) = Self::getTitleID(&content)?;

        // Get price.
        let price_url = "https://api.ec.nintendo.com/v1/price";
        let req = reqwest::Client::new().get(price_url).query(
            &[("country", self.country), ("lang", self.lang), ("ids", real_id)]);
        let content = utils::queryRequest(req).await?;
        let data: json::Value = serde_json::from_str(&content).map_err(
            |_| rterr!("Failed to parse JSON"))?;

        let price_data = &data["prices"][0]["regular_price"];
        let price_raw = price_data["raw_value"].as_str().ok_or(
            rterr!("Failed to get price"))?;
        let price = utils::parsePrice(&price_raw)?;
        let price_str = price_data["amount"].as_str().ok_or(
            rterr!("Failed to get price string"))?;
        let mut item = ItemInfo::new(self.name, id);
        item.name = title.to_owned();
        item.url = store_url;
        item.price = price;
        item.price_str = price_str.to_owned();
        Ok(item)
    }
}
