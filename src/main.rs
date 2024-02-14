use std::{path::Path, process::exit, time::Duration};

use chrono::Utc;
use clap::Parser;
use models::{area_information::AreaInformation, args::Args};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use crate::models::{area_information::Event, config::Config};

mod models;

mod logging {
    use chrono::Utc;
    pub fn log(log_line: String) {
        println!("{} | {}", Utc::now().format("%d-%m-%Y %H:%M:%S"), log_line);
    }
}

static CONFIG_FILE_PATH: &'static str = "esp-scheduler.yaml";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();
    let config_file_path = args.config_file.as_ref().map_or(CONFIG_FILE_PATH, |v| v);

    if args.init {
        if Path::new(config_file_path).exists() {
            logging::log(format!("config file already exists: {}", config_file_path));
            exit(1)
        } else {
            let default_config_str = serde_yaml::to_string(&Config::default())?;
            let mut config_file = File::create(config_file_path).await?;
            config_file.write_all(default_config_str.as_bytes()).await?;
        }
        exit(0)
    }

    logging::log(format!("starting esp-scheduler"));
    logging::log(format!("using configuration file: {}", config_file_path));
    let mut file = File::open(config_file_path)
        .await
        .expect("Could not open config file");
    let mut file_contents = String::new();

    file.read_to_string(&mut file_contents).await?;

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
            if let Some(source_from_file) = args.source_from_file.as_ref() {
                let source_from_file_path = Path::new(source_from_file);
                if source_from_file_path.exists() {
                    logging::log(format!("using {} as event source", source_from_file));
                    let mut source_file = File::open(source_from_file_path).await?;
                    let mut source_json = String::new();
                    source_file.read_to_string(& mut source_json).await?;
                    cached_body = source_json;
                } else {
                    logging::log(format!("{} does not exist", source_from_file));
                    exit(1);
                }
            } else {

                logging::log(format!("{}, {}", "fetching", &configuration.fetch.url));
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
                    logging::log(format!("{}\n{}", "Unable to reach host", err));
                    continue;
                }
                let text = response?.text().await;
                if let Err(err) = text {
                    logging::log(format!(
                        "{}\n{}",
                        "Could not read body of http response", err
                    ));
                    continue;
                }
                cached_body = text.unwrap();
            }
        }

        let area_information = AreaInformation::new(&cached_body)?;

        let mut events: Vec<&Event> = area_information
            .events
            .iter()
            .filter(|&e| e.start.to_utc() > Utc::now())
            .collect();

        events.sort_by_key(|a| a.start);

        if let Some(&first) = events.first() {
            let time_diff = first.start.to_utc() - Utc::now();

            logging::log(format!(
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
                logging::log(format!(
                    "{}: {} mins",
                    "threshold reached", configuration.threshold
                ));
                let commands = configuration
                    .commands
                    .as_ref() // convert from &Option<T> to Option<&T>
                    .map(|v| &v[..]) // map to slice instead of a container reference
                    .unwrap_or_else(|| &[]); // otherwise return an empty slice.
                for command in commands {
                    logging::log(format!("executing command: {}", command));
                    let child = Command::new("sh")
                        .args(["-c", command])
                        .current_dir(".")
                        .spawn()?
                        .wait()
                        .await;
                    if let Err(err) = child {
                        logging::log(format!("{}\n{}", "could not execute command", err));
                    }
                }
            }
        } else {
            println!("no upcoming events");
        }

        count += 1;
    }
}
