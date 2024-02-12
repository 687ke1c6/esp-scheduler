use std::{fs::File, io::Read, time::Duration};

use chrono::Utc;
use models::area_information::AreaInformation;
use tokio::process::Command;

use crate::models::config::Config;

mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("esp-scheduler.yaml").expect("Could not open config file");
    let mut file_contents = String::new();

    File::read_to_string(&mut file, &mut file_contents).unwrap();

    let configuration =
        Config::from_str(&file_contents).expect("Could not deserialize configuration");

    let interval_secs = u64::from(configuration.interval_mins.as_ref().unwrap_or(&1) * 60);

    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    let http_client = reqwest::Client::new();
    let mut count = 0;
    let mut cached_body = String::from("{\"events\" : []}");
    let fetch_mins = configuration.fetch_occurrence.unwrap_or_else(|| 60);

    loop {
        interval.tick().await;
        if count % fetch_mins == 0 {
            println!("{}, {}", "fetching", &configuration.fetch.url);
            let mut request = http_client.get(&configuration.fetch.url);

            // apply headers
            if let Some(headers) = configuration.fetch.headers.as_ref() {
                for (key, value) in headers {
                    request = request.header(key, value);
                }
            }

            // send reqwest
            let response = request.send().await;
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

        let mut events = area_information.events;
        events.sort_by_key(|a| a.start);

        if let Some(first) = events.first() {
            let time_diff = first.start.to_utc() - Utc::now();
            println!("{} in {} mins", first.note, time_diff.num_minutes());
            if time_diff.num_minutes() < 0 {
                println!(
                    "WARNING: {} mins left\n\tnext: {}\n\tnow: {}",
                    time_diff.num_minutes(),
                    first.start.to_utc(),
                    Utc::now()
                );
            } else if time_diff.num_minutes() < configuration.threshold.into() {
                println!("{}: {} mins", "threshold reached", configuration.threshold);
                let commands = configuration
                    .commands
                    .as_ref() // convert from &Option<T> to Option<&T>
                    .map(|v| &v[..]) // map to slice instead of a container reference
                    .unwrap_or_else(|| &[]); // otherwise return an empty slice.
                for command in commands {
                    println!("executing command: {}", command);
                    let child = Command::new("sh")
                        .args(["-c", command])
                        .current_dir(".")
                        .spawn()?
                        .wait()
                        .await;
                    if let Err(err) = child {
                        println!("{}\n{}", "could not execute command", err);
                    }
                }
            }
        }

        count += 1;
    }
}
