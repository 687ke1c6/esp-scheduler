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
    pub fn default() -> Self {
        Config {
            interval_mins: Some(5),
            fetch_occurrence: Some(12),
            threshold: 10,
            fetch: Fetch {
                url: "".to_string(),
                headers: None
            },
            commands: Some(vec!["cat /etc/hostname".to_string(), "date".to_string()])
        }
    }
}