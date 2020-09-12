#![allow(non_snake_case)]

use std::str::FromStr;
use std::path::PathBuf;
use std::process::exit;

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

fn main()
{
    stderrlog::new().verbosity(2).init().unwrap();

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
        .get_matches();

    match opts.subcommand_name()
    {
        Some("serve") =>
        {
            let conf = match loadConfig(opts.value_of("config"))
            {
                Ok(conf) => conf,
                Err(e) => { log_error!("{}", e); exit(2); },
            };

            info!("Wishlist service starting...");
            web::start(conf.url_prefix);
            exit(0);
        },

        None =>
        {
            println!("{}", opts.usage());
            exit(0);
        },

        Some(a) =>
        {
            log_error!("Unknown command: {}", a);
            exit(-1);
        },
    }
}
