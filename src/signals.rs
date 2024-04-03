use futures_util::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::process::exit;

use crate::logging;

pub async fn handle_signals() {
    let mut signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT]).expect("Could not create signals");
    let handle = signals.handle();
    while let Some(signal) = signals.next().await {
        match signal {
            SIGHUP => {
                // Reload configuration
                // Reopen the log file
            }
            SIGTERM | SIGINT | SIGQUIT => {
                // Shutdown the system;
                logging::log(format!("closing"));
                handle.close();
                exit(1);
            }
            _ => unreachable!(),
        }
    }
}