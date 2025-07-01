use clap::Parser;
use scout_rs::client::ScoutClient;
use std::path::Path;
use tracing::{ info, error };
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "upload_batch")]
#[command(about = "Upload a directory of images to Scout using batch uploads")]
struct Args {
    /// Directory containing images to upload
    #[arg(short, long)]
    directory: String,

    /// EarthRanger URL
    #[arg(short, long)]
    earthranger_url: Option<String>,

    /// Whether the events should be public
    #[arg(short, long, default_value = "false")]
    is_public: bool,

    /// Message to include with events
    #[arg(short, long)]
    message: Option<String>,

    /// Default latitude for files without metadata
    #[arg(long)]
    default_latitude: Option<f64>,

    /// Default longitude for files without metadata
    #[arg(long)]
    default_longitude: Option<f64>,

    /// Default altitude for files without metadata
    #[arg(long)]
    default_altitude: Option<f64>,

    /// Default heading for files without metadata
    #[arg(long)]
    default_heading: Option<f64>,

    /// Batch size for uploads (max 50)
    #[arg(short, long, default_value = "10")]
    batch_size: usize,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

const MAX_BATCH_SIZE: usize = 50;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt().with_env_filter(format!("scout_rs={}", args.log_level)).init();

    // Validate batch size
    if args.batch_size == 0 || args.batch_size > MAX_BATCH_SIZE {
        error!("Batch size must be between 1 and {}", MAX_BATCH_SIZE);
        std::process::exit(1);
    }

    // Check if directory exists
    if !Path::new(&args.directory).exists() {
        error!("Directory does not exist: {}", args.directory);
        std::process::exit(1);
    }

    // Get API key from environment
    let api_key = std::env
        ::var("SCOUT_API_KEY")
        .expect("SCOUT_API_KEY environment variable must be set");
    let scout_url = std::env
        ::var("SCOUT_URL")
        .unwrap_or_else(|_| "http://localhost:3000/api/scout".to_string());

    info!("🚀 Starting batch upload");
    info!("   Directory: {}", args.directory);
    info!("   Batch size: {}", args.batch_size);
    info!("   EarthRanger URL: {:?}", args.earthranger_url);
    info!("   Public: {}", args.is_public);

    // Create client
    let mut client = ScoutClient::new(scout_url, api_key)?;

    // Identify device and herd
    if let Err(e) = client.identify().await {
        error!("Failed to identify device: {}", e);
        std::process::exit(1);
    }

    // Perform batch upload
    let result = client.upload_directory_batch(
        &args.directory,
        args.earthranger_url.as_deref(),
        args.is_public,
        args.message.as_deref(),
        args.default_latitude,
        args.default_longitude,
        args.default_altitude,
        args.default_heading,
        args.batch_size
    ).await?;

    // Print results
    result.print_summary();

    if result.failed_uploads > 0 {
        std::process::exit(1);
    }

    Ok(())
}
