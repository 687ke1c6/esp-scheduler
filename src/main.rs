use std::{fs::File, io::Read, time::Duration};

use chrono::Utc;
use models::area_information::AreaInformation;
use tokio::process::Command;

use crate::models::config::Config;

mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let mut file = File::open("config.yaml").unwrap();
    let mut file_contents = String::new();
    
    File::read_to_string(&mut file, &mut file_contents).unwrap();
    
    let configuration = Config::from_str(&file_contents).expect("Could not deserialize config.yaml");

    let mut interval = tokio::time::interval(Duration::from_secs(u64::from(configuration.interval_mins)));
    let http_client = reqwest::Client::new();
    let mut count = 0;
    let mut cached_body = String::from("{\"events\" : []}");

    loop {
        interval.tick().await;
        if count % configuration.fetch_occurrence == 0 {
            println!("{}, {}", "fetching", &configuration.url);
            let response = http_client
                .get(&configuration.url)
                .header("token", &configuration.token)
                .send()
                .await;
            if let Err(err) = response {
                println!("{}\n{}", "Unable to reach host", err);
                continue;
            }
            let text = response.unwrap().text().await;
            if let Err(err) = text {
                println!("{}\n{}", "Could not read body of http response", err);
                continue;
            }
            cached_body = text.unwrap();
        }

        let area_information = AreaInformation::new(&cached_body)?;
        // println!("{:?}", area_information);

        let mut events = area_information.events;
        events.sort_by_key(|a| a.start);

        if let Some(first) = events.first() {
            let time_diff = first.start.to_utc() - Utc::now();
            println!(
                "{} in {} mins",
                first.note,
                time_diff.num_minutes()
            );
            if time_diff.num_minutes() < configuration.execute_within_mins.into() {
                for command in &configuration.command {
                    let child = Command::new("sh")
                        .args(["-c", command])
                        .current_dir(".")
                        .spawn();
                    if let Err(err) = child {
                        println!("{}\n{}", "could not execute command", err);
                    }
                }
            }
        }

        count += 1;
    }
}
