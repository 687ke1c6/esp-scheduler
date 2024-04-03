use std::collections::HashMap;

use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct FetchResponse {
    pub code: u16,
    pub content: String
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Fetch {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub response: Option<FetchResponse>
}

impl Fetch {
    pub fn to_reqwest(&self, client: &Client) -> RequestBuilder {
        let mut request = client.get(&self.url);

        // apply headers
        if let Some(headers) = &self.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }
        request
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub version: Option<u8>,
    pub interval_mins: Option<u32>,
    pub fetch_occurrence: Option<u32>,
    pub threshold: u32,
    pub fetch: Option<Fetch>,
    pub commands: Option<Vec<String>>,
}

impl Config {
    pub fn from_yaml_str(yaml_str: &str) -> Result<Config, serde_yaml::Error> {
        serde_yaml::from_str::<Config>(yaml_str)
    }
    pub fn default() -> Self {
        Config {
            version: Some(2),
            interval_mins: Some(5),
            fetch_occurrence: Some(12),
            threshold: 10,
            fetch: Some(Fetch {
                url: "".to_string(),
                headers: None,
                response: None
            }),
            commands: Some(vec!["cat /etc/hostname".to_string(), "date".to_string()])
        }
    }
}