use serde_json as json;

use crate::error::Error;
use crate::store::ItemInfo;
use crate::utils;

pub enum Region
{
    US,
}

pub struct Epic
{
    region: &'static str,
    name: &'static str,
}

impl Epic
{
    pub fn new(region: Region) -> Self
    {
        match region
        {
            Region::US => Self{ region: "en-US", name: "epic-us" },
        }
    }

    fn dataURL(&self, id: &str) -> String
    {
        format!("https://epicgames.com/store/{}/p/{}", self.region, id)
    }

    pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
    {
        // The items in Epic store are very well-defined. The info is
        // embeded in the HTML as Linked Data JSON.
        let url = self.dataURL(id);
        let content = utils::get(&url).await?;
        let (s, _, _) = utils::findSubStr(
            &content, "type=\"application/ld+json\"", "</script>").ok_or_else(
            || rterr!("Invalid Epic item page"))?;
        let begin = s.find("{").ok_or_else(
            || rterr!("Failed to find beginning of JSON"))?;
        let json_str = &s[begin..];

        let data: json::Value = serde_json::from_str(json_str).map_err(
            |_| rterr!("Failed to parse JSON"))?;
        let price = data["offers"][0]["priceSpecification"]["price"].as_f64()
            .ok_or_else(|| rterr!("Failed to extract price"))?;
        let name = data["name"].as_str().ok_or_else(
            || rterr!("Failed to extract name"))?;

        let mut item = ItemInfo::new(self.name, id);
        item.name = name.to_owned();
        item.price = (price * 100.0) as i64;
        item.price_str = format!("${}", price);
        item.url = url;
        Ok(item)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use tokio;

    #[test]
    fn get_price() -> Result<(), Error>
    {
        let a = Epic::new(Region::US);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let item = rt.block_on(a.get("hitman-3"))?;
        assert!(item.name.find("HITMAN 3").is_some());
        assert!(item.price > 0);
        Ok(())
    }
}
