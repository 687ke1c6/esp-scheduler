use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Fetch {
    pub url: String,
    pub headers: Option<HashMap<String, String>>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub interval_mins: Option<u32>,
    pub fetch_occurrence: Option<u32>,
    pub threshold: u32,
    pub fetch: Fetch,
    pub commands: Option<Vec<String>>,
}

impl Config {
    pub fn from_str(str: &str) -> Result<Config, serde_yaml::Error> {
        serde_yaml::from_str::<Config>(str)
    }
}