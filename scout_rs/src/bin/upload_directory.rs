use clap::Parser;
use scout_rs::client::ScoutClient;
use std::env;
use tracing::{ info, warn };

#[derive(Parser, Debug)]
#[command(author, version, about = "Upload a directory of images to Scout", long_about = None)]
struct Args {
    /// Directory path containing images to upload
    #[arg(short, long)]
    directory: String,

    /// Scout URL
    #[arg(long, default_value = "http://localhost:3000/api/scout")]
    scout_url: String,

    /// API Key (or set SCOUT_DEVICE_API_KEY env var)
    #[arg(long)]
    api_key: Option<String>,

    /// EarthRanger URL
    #[arg(long)]
    earthranger_url: Option<String>,

    /// Make events public
    #[arg(long, default_value = "false")]
    public: bool,

    /// Message to include with events
    #[arg(short, long)]
    message: Option<String>,

    /// Default latitude for files that can't be parsed
    #[arg(long)]
    default_latitude: Option<f64>,

    /// Default longitude for files that can't be parsed
    #[arg(long)]
    default_longitude: Option<f64>,

    /// Default altitude for files that can't be parsed
    #[arg(long)]
    default_altitude: Option<f64>,

    /// Default heading for files that can't be parsed
    #[arg(long)]
    default_heading: Option<f64>,

    /// Batch size for uploads (max 50, default: 20)
    #[arg(long, default_value = "20")]
    batch_size: usize,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

// Example usage:
// cargo run --bin upload_directory -- --directory /path/to/photos --api-key your_api_key
// cargo run --bin upload_directory -- --directory /path/to/photos --default-latitude 19.754824 --default-longitude -155.15393

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt().with_env_filter(format!("scout_rs={}", args.log_level)).init();

    // Get API key from args or environment
    let api_key = args.api_key.unwrap_or_else(|| {
        env::var("SCOUT_DEVICE_API_KEY").expect(
            "SCOUT_DEVICE_API_KEY environment variable not set or --api-key not provided"
        )
    });

    info!("🚀 Starting directory upload to Scout...");
    info!("   Scout URL: {}", args.scout_url);
    info!("   Directory: {}", args.directory);

    if let Some(url) = &args.earthranger_url {
        info!("   EarthRanger URL: {}", url);
    }
    info!("   Public: {}", args.public);

    if let Some(msg) = &args.message {
        info!("   Message: {}", msg);
    }
    if let Some(lat) = args.default_latitude {
        info!("   Default latitude: {}", lat);
    }
    if let Some(lon) = args.default_longitude {
        info!("   Default longitude: {}", lon);
    }
    if let Some(alt) = args.default_altitude {
        info!("   Default altitude: {}", alt);
    }
    if let Some(hdg) = args.default_heading {
        info!("   Default heading: {}", hdg);
    }
    info!("   Batch size: {}", args.batch_size);

    // Create Scout client
    let mut client = ScoutClient::new(args.scout_url, api_key)?;

    // Identify and load device/herd information into state
    client.identify().await?;

    // Upload directory (device ID will be automatically retrieved from stored state)
    // TODO: Implement upload_directory method in ScoutClient
    /*
    let result = client.upload_directory(
        &args.directory,
        args.earthranger_url.as_deref(),
        args.public,
        args.message.as_deref(),
        args.default_latitude,
        args.default_longitude,
        args.default_altitude,
        args.default_heading,
        Some(args.batch_size)
    ).await?;

    // Print results
    result.print_summary();

    if result.failed_uploads > 0 {
        warn!("⚠️  Some uploads failed. Check the failed files list above.");
        std::process::exit(1);
    } else {
        info!("✅ All uploads completed successfully!");
    }
    */

    info!("⚠️  upload_directory method not yet implemented in new client");
    info!("   This binary is temporarily disabled during the transition to the new API");

    Ok(())
}
