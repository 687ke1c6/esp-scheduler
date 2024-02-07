use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub interval_mins: u32,
    pub fetch_occurrence: u32,
    pub execute_within_mins: u32,
    pub url: String,
    pub command: Vec<String>,
    pub token: String
}

impl Config {
    pub fn from_str(str: &str) -> Result<Config, serde_yaml::Error> {
        serde_yaml::from_str::<Config>(str)
    }
}