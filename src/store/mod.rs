use serde::{Deserialize, Serialize};

use crate::error::Error;
mod playstation;
mod switch;

/// An abstraction for the info one get when querying a store.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemInfo
{
    pub name: String,
    pub store: String,
    pub id: String,
    pub url: String,
    pub price: i64,                 // Price * 100
    pub price_str: String,
}

#[allow(dead_code)]
impl ItemInfo
{
    pub fn new(store: &str, id: &str) -> Self
    {
        Self{ name: String::new(),
              store: store.to_owned(),
              id: id.to_owned(),
              url: String::new(),
              price: 0,
              price_str: String::new(),
        }
    }

    pub fn bare(&self) -> bool
    {
        self.name.is_empty()
    }
}

macro_rules! store_enum
{
    {
        $( $sub_name:ident($store_type:ty),)+
    } => {
        pub enum Store
        {
            $( $sub_name($store_type), )+
        }

        impl Store
        {
            pub async fn get(&self, id: &str) -> Result<ItemInfo, Error>
            {
                match self
                {
                    $( Self::$sub_name(s) => s.get(id).await, )+
                }
            }
        }
    }
}

store_enum!
{
    PlayStation(playstation::PlayStation),
    Switch(switch::Switch),
}

impl Store
{
    pub fn new(store_name: &str) -> Result<Self, Error>
    {
        match store_name
        {
            "ps4-us" => Ok(Self::PlayStation(
                playstation::PlayStation::new(playstation::Region::US))),
            "ps4-hk" => Ok(Self::PlayStation(
                playstation::PlayStation::new(playstation::Region::HK))),
            "switch-us" => Ok(Self::Switch(
                switch::Switch::new(switch::Region::US))),
            _ => Err(rterr!("Invalid store: {}", store_name)),
        }
    }
}
