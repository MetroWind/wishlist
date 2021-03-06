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

    fn dataURL(&self, id: &str) -> String
    {
        format!("https://store.playstation.com/valkyrie-api/{}/19/resolve/{}",
                self.region, id)
    }

    pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
    {
        let url = self.dataURL(id);
        let content = utils::get(&url).await?;
        let data: json::Value = serde_json::from_str(&content).map_err(
            |_| rterr!("Failed to parse JSON"))?;

        for item in data["included"].as_array().ok_or(
            rterr!("Failed to get sub-item"))?
        {
            if let Some(game_type) = item["attributes"]["game-content-type"].as_str()
            {
                if game_type == "Full Game" || game_type == "PSN Game"
                {
                    let name = item["attributes"]["name"].as_str().ok_or(
                        rterr!("Failed to get item name"))?;
                    let price_obj = &item["attributes"]["skus"][0]["prices"]
                        ["non-plus-user"]["actual-price"];
                    let price = price_obj["value"].as_i64().ok_or(
                        rterr!("Failed to get price"))?;
                    let price_str = price_obj["display"].as_str().ok_or(
                        rterr!("Failed to get price string"))?;
                    let mut item = ItemInfo::new(self.name, id);
                    item.name = name.to_owned();
                    item.url = format!(
                        "https://store.playstation.com/en-us/product/{}", id);
                    item.price = price;
                    item.price_str = price_str.to_owned();
                    return Ok(item);
                }
            }
        }
        Err(rterr!("Failed to retrieve item info"))
    }
}
