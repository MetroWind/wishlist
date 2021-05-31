#![allow(non_snake_case)]

use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use log::{debug,info,warn};
use log::error as log_error;
use stderrlog;
use clap;

#[macro_use]
mod error;
mod store;
mod utils;
mod web;
mod data;
mod config;
mod middle;

use crate::error::Error;

const CONF_FILE: &str = "wishlist.toml";

fn readConfig(path: &std::path::Path) -> Result<config::ConfigParams, Error>
{
    debug!("Reading config from {}...", path.to_string_lossy());
    config::ConfigParams::fromFile(path)
}

fn findConfigFile() -> Option<PathBuf>
{
    // Current dir
    let path = PathBuf::from_str(CONF_FILE).unwrap();
    if path.exists()
    {
        return Some(path);
    }

    // /etc
    if let Ok(mut path) = PathBuf::from_str("/etc")
    {
        path.push(CONF_FILE);
        if path.exists()
        {
            return Some(path);
        }
    }
    None
}

fn loadConfig(specified: Option<&str>) -> Result<config::ConfigParams, Error>
{
    let conf_file = if let Some(config) = specified
    {
        if let Ok(path) = PathBuf::from_str(config)
        {
            path
        }
        else
        {
            log_error!("Invalid config path: {}", config);
            exit(1);
        }
    }
    else if let Some(path) = findConfigFile()
    {
        path
    }
    else
    {
        warn!("Config file not found. Using default parameters...");
        return Ok(config::ConfigParams::default());
    };

    readConfig(&conf_file)
}

fn realMain() -> Result<(), Error>
{
    let opts = clap::App::new("Wishlist service")
        .version("0.1")
        .author("MetroWind <chris.corsair@gmail.com>")
        .arg(clap::Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Location of config file. Default: wishlist.toml or /etc/wishlist.toml")
             .takes_value(true))
        .subcommand(clap::App::new("serve")
                    .about("Start API server"))
        .subcommand(clap::App::new("add")
                    .about("Add an item")
                    .arg(clap::Arg::with_name("store")
                         .required(true)
                         .help("Store name"))
                    .arg(clap::Arg::with_name("id")
                         .required(true)
                         .help("Item ID")))
        .subcommand(clap::App::new("remove")
                    .about("Remove an item and its price history")
                    .arg(clap::Arg::with_name("store")
                         .required(true)
                         .help("Store name"))
                    .arg(clap::Arg::with_name("id")
                         .required(true)
                         .help("Item ID")))
        .subcommand(clap::App::new("list")
                    .about("Print all items"))
        .subcommand(clap::App::new("update")
                    .about("Update price of all items"))
        .get_matches();

    match opts.subcommand_name()
    {
        Some("serve") =>
        {
            let conf = loadConfig(opts.value_of("config"))?;
            middle::maybeInitDB(&conf)?;
            let server = web::WebHandler::new(&conf);
            info!("Wishlist service starting...");
            server.start();
        },

        Some("add") =>
        {
            let conf = loadConfig(opts.value_of("config"))?;
            let subopts = opts.subcommand_matches("add").unwrap();
            middle::addItem(subopts.value_of("store").unwrap(),
                            subopts.value_of("id").unwrap(), conf)?;
        },
        Some("remove") =>
        {
            let subopts = opts.subcommand_matches("remove").unwrap();
            let conf = loadConfig(opts.value_of("config"))?;
            middle::removeItem(subopts.value_of("store").unwrap(),
                               subopts.value_of("id").unwrap(), conf)?;
        },
        Some("list") =>
        {
            let conf = loadConfig(opts.value_of("config"))?;
            middle::listItems(conf)?;
        },
        Some("update") =>
        {
            let conf = loadConfig(opts.value_of("config"))?;
            middle::updateItemPrices(conf)?;
        },
        None =>
        {
            println!("{}", opts.usage());
        },

        Some(a) =>
        {
            return Err(rterr!("Unknown command: {}", a));
        },
    }
    Ok(())
}

fn main()
{
    stderrlog::new().module(module_path!()).verbosity(3).init().unwrap();
    if let Err(e) = realMain()
    {
        log_error!("{}", e);
        exit(1);
    }
}
