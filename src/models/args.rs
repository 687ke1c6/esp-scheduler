use clap::Parser;

/// Eskom Se Push command scheduler
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Initialize configuration file
    #[arg(short, long)]
    pub init: bool,
    /// Configuration file path
    #[arg(short, long)]
    pub config_file: Option<String>,
    /// Start delay in seconds
    #[arg(short, long)]
    pub delay: Option<u32>
}