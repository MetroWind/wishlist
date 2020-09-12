use serde_json as json;

use crate::error::Error;
use crate::store::ItemInfo;
use crate::utils;

pub enum Region
{
    US,
    HK,
}

pub struct PlayStation
{
    region: &'static str,
    name: &'static str,
}

impl PlayStation
{
    pub fn new(region: Region) -> Self
    {
        match region
        {
            Region::US => Self{ region: "en/US", name: "ps4-us" },
            Region::HK => Self{ region: "zh/HK", name: "ps4-hk" },
        }
    }

    fn url(&self, id: &str) -> String
    {
        format!("https://store.playstation.com/valkyrie-api/{}/19/resolve/{}",
                self.region, id)
    }

    pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
    {
        let url = self.url(id);
        let content = utils::get(&url).await?;
        let data: json::Value = serde_json::from_str(&content).map_err(
            |_| error!(RuntimeError, "Failed to parse JSON"))?;

        for item in data["included"].as_array().ok_or(
            error!(RuntimeError, "Failed to get sub-item"))?
        {
            if let Some(game_type) = item["attributes"]["game-content-type"].as_str()
            {
                if game_type == "Full Game"
                {
                    let name = item["attributes"]["name"].as_str().ok_or(
                        error!(RuntimeError, "Failed to get item name"))?;
                    let price_obj = &item["attributes"]["skus"][0]["prices"]["non-plus-user"]["actual-price"];
                    let price = price_obj["value"].as_i64().ok_or(
                        error!(RuntimeError, "Failed to get price"))?;
                    let price_str = price_obj["display"].as_str().ok_or(
                        error!(RuntimeError, "Failed to get price string"))?;
                    return Ok(ItemInfo{
                        name: name.to_owned(),
                        store: self.name.to_owned(),
                        id: id.to_owned(),
                        url: url,
                        price: price,
                        price_str: price_str.to_owned() });
                }
            }
        }
        Err(error!(RuntimeError, "Failed to retrieve item info"))
    }
}
