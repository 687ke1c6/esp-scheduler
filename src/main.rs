use std::{fs::File, io::Read, time::Duration};

use chrono::Utc;
use models::area_information::AreaInformation;
use tokio::process::Command;

use crate::models::{area_information::Event, config::Config};

mod models;

fn log(log_line: String) {
    println!("{} | {}", Utc::now().format("%d-%m-%Y %H:%M:%S"), log_line);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log(format!("starting esp-scheduler"));
    let mut file = File::open("esp-scheduler.yaml").expect("Could not open config file");
    let mut file_contents = String::new();

    File::read_to_string(&mut file, &mut file_contents).unwrap();

    let configuration =
        Config::from_str(&file_contents).expect("Could not deserialize configuration");

    let interval_secs = u64::from(configuration.interval_mins.as_ref().unwrap_or(&1) * 60);
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    let http_client = reqwest::Client::new();
    let mut cached_body = String::from("{\"events\" : []}");
    let fetch_mins = configuration.fetch_occurrence.unwrap_or_else(|| 60);

    let mut count = 0;
    loop {
        interval.tick().await;
        if count % fetch_mins == 0 {
            log(format!("{}, {}", "fetching", &configuration.fetch.url));
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
                log(format!("{}\n{}", "Unable to reach host", err));
                continue;
            }
            let text = response.unwrap().text().await;
            if let Err(err) = text {
                log(format!(
                    "{}\n{}",
                    "Could not read body of http response", err
                ));
                continue;
            }
            cached_body = text.unwrap();
        }

        let area_information = AreaInformation::new(&cached_body)?;

        let mut events: Vec<&Event> = area_information
            .events
            .iter()
            .filter(|&e| e.start.to_utc() > Utc::now())
            .collect();

        events.sort_by_key(|a| a.start);

        if let Some(first) = events.first() {
            let time_diff = first.start.to_utc() - Utc::now();

            log(format!(
                "{} in {} mins. {}",
                first.note,
                time_diff.num_minutes(),
                if time_diff.num_hours() <= 1 {
                    format!("")
                } else {
                    format!("(~{} hrs)", time_diff.num_hours())
                }
            ));
            if time_diff.num_minutes() < configuration.threshold.into() {
                log(format!(
                    "{}: {} mins",
                    "threshold reached", configuration.threshold
                ));
                let commands = configuration
                    .commands
                    .as_ref() // convert from &Option<T> to Option<&T>
                    .map(|v| &v[..]) // map to slice instead of a container reference
                    .unwrap_or_else(|| &[]); // otherwise return an empty slice.
                for command in commands {
                    log(format!("executing command: {}", command));
                    let child = Command::new("sh")
                        .args(["-c", command])
                        .current_dir(".")
                        .spawn()?
                        .wait()
                        .await;
                    if let Err(err) = child {
                        log(format!("{}\n{}", "could not execute command", err));
                    }
                }
            }
        }

        count += 1;
    }
}
