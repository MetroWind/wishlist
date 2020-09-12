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

    fn url(&self, id: &str) -> String
    {
        format!("https://www.nintendo.com/games/detail/{}/", id)
    }

    pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
    {
        // Get title and real ID.
        let store_url = self.url(id);
        let content = utils::get(&store_url).await?;

        // Get position of ‘{’.
        let (store_data_str, _, _) = utils::findSubStr(
            &content, "Object.freeze({", "});").ok_or(
            error!(RuntimeError, "Failed to find store data"))?;

        let (_, title_json_begin, title_json_end) = utils::findSubStr(
            store_data_str, "title: ", "\",")
            .ok_or(error!(RuntimeError, "Failed to find store data"))?;

        let title: String = serde_json::from_str(
            &store_data_str[title_json_begin..title_json_end+1]).map_err(
            |_| error!(RuntimeError, "Failed to parse title JSON"))?;
        // let title = title_data.as_str().ok_or(
        //     error!(RuntimeError, "Failed to get item name"))?;

        let (real_id, _, _) = utils::findSubStr(store_data_str, "nsuid: \"", "\",")
            .ok_or(error!(RuntimeError, "Failed to find real ID"))?;

        // Get price.
        let price_url = "https://api.ec.nintendo.com/v1/price";
        let req = reqwest::Client::new().get(price_url).query(
            &[("country", self.country), ("lang", self.lang), ("ids", real_id)]);
        let content = utils::queryRequest(req).await?;
        let data: json::Value = serde_json::from_str(&content).map_err(
            |_| error!(RuntimeError, "Failed to parse JSON"))?;

        let price_data = &data["prices"][0]["regular_price"];
        let price_raw = price_data["raw_value"].as_str().ok_or(
            error!(RuntimeError, "Failed to get price"))?;
        if price_raw.find(".").is_none()
        {
            return Err(error!(RuntimeError, "Invalid price format"));
        }
        // “39.99” -> 3999
        let price: i64 = price_raw.replace(".", "").parse().map_err(
            |_| error!(RuntimeError, "Failed to parse price"))?;
        let price_str = price_data["amount"].as_str().ok_or(
            error!(RuntimeError, "Failed to get price string"))?;
        return Ok(ItemInfo{
            name: title.to_owned(),
            store: self.name.to_owned(),
            id: id.to_owned(),
            url: store_url,
            price: price,
            price_str: price_str.to_owned() });
    }
}
