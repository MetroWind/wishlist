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
            error!(RuntimeError, "Failed to find store data"))?;

        let (title, _, _) = utils::findSubStr(
            store_data_str, "title: \"", "\",")
            .ok_or(error!(RuntimeError, "Failed to find store data"))?;

        // let title: String = serde_json::from_str(
        //     &store_data_str[title_json_begin..title_json_end+1]).map_err(
        //     |_| error!(RuntimeError, "Failed to parse title JSON"))?;

        let (real_id, _, _) = utils::findSubStr(store_data_str, "nsuid: \"", "\",")
            .ok_or(error!(RuntimeError, "Failed to find real ID"))?;
        Ok((title, real_id))
    }

    /// “39.99” -> 3999, “15” -> 1500
    fn parsePrice(price_raw: &str) -> Result<i64, Error>
    {
        if price_raw.find(".").is_none()
        {
            price_raw.parse::<i64>().map_err(
                |_| error!(RuntimeError,
                           format!("Failed to parse price: {}", price_raw)))
                .map(|x| x * 100)
        }
        else
        {
            price_raw.replace(".", "").parse().map_err(
                |_| error!(RuntimeError,
                           format!("Failed to parse price: {}", price_raw)))
        }
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
            |_| error!(RuntimeError, "Failed to parse JSON"))?;

        let price_data = &data["prices"][0]["regular_price"];
        let price_raw = price_data["raw_value"].as_str().ok_or(
            error!(RuntimeError, "Failed to get price"))?;
        let price = Self::parsePrice(&price_raw)?;
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
