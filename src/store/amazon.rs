use std::time::Duration;

use scraper::{Html, Selector};

use crate::error::Error;
use crate::store::ItemInfo;
use crate::utils;

pub struct Amazon
{
    name: &'static str,
}

impl Amazon
{
    pub fn new() -> Self
    {
        Self{ name: "amazon-us" }
    }

    fn parsePriceTR(tr: scraper::element_ref::ElementRef) -> Option<&str>
    {
        if let Some(node_id) = tr.value().attr("id")
        {
            if !node_id.starts_with("priceblock_")
            {
                return None;
            }
        }
        else
        {
            return None;
        }

        // We have found a price tr.
        let sel_td = Selector::parse("td").unwrap();
        for td in tr.select(&sel_td)
        {
            if let Some(price_s) = Self::parsePriceTD(td)
            {
                return Some(price_s);
            }
        }
        None
    }

    fn parsePriceTD(td: scraper::element_ref::ElementRef) -> Option<&str>
    {
        // The price span seems to be always in a td with class
        // “a-span12”.
        if let Some(c) = td.value().attr("class")
        {
            if !c.starts_with("a-span")
            {
                return None;
            }
        }
        else
        {
            return None;
        }

        let sel_span = Selector::parse("span").unwrap();
        for span in td.select(&sel_span)
        {
            if let Some(id) = span.value().attr("id")
            {
                if id.starts_with("priceblock_")
                {
                    return span.text().next();
                }
            }
        }
        None
    }

    pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
    {
        let url = format!("https://amazon.com/gp/product/{}/", id);
        let content: String = utils::get(&url).await?;

        let document = Html::parse_document(&content);

        // Extract price
        let sel_price = Selector::parse("#price tr").unwrap();
        let mut price_str = String::new();
        for node in document.select(&sel_price)
        {
            if let Some(price_s) = Self::parsePriceTR(node)
            {
                price_str = price_s.to_owned();
                break;
            }
        }
        if price_str.is_empty()
        {
            return Err(rterr!("Failed to get amazon price of {}", id));
        }
        if !price_str.starts_with("$")
        {
            return Err(rterr!("Invalid amazon price for {}: {}", id, price_str));
        }

        let price = utils::parsePrice(&price_str[1..])?;

        // Extract name
        let sel_name = Selector::parse("#productTitle").unwrap();
        let name_node = document.select(&sel_name).next().ok_or_else(
            || rterr!("Failed to extract amazon name of {}", id))?;
        let name = name_node.text().next().ok_or_else(
            || rterr!("Amazon name node does not contain a name for product {}",
                      id))?;

        let mut item = ItemInfo::new(self.name, id);
        item.name = name.trim().to_owned();
        item.url = url;
        item.price = price;
        item.price_str = price_str;
        item.update_interval = Some(Duration::from_secs(600));
        Ok(item)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use tokio;

    type AnyError = Box<dyn std::error::Error>;

    #[test]
    fn get_price() -> Result<(), Error>
    {
        let a = Amazon::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let item = rt.block_on(a.get("B08164VTWH"))?;
        assert!(item.name.find("5900X").is_some());
        assert!(item.price > 0);
        Ok(())
    }
}
