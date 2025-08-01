use clap::Parser;
use serde_json;
use std::env;
use scout_rs::client::{ ScoutClient, Event, Tag, ResponseScoutStatus };

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, rename_all = "snake_case")]
struct Args {
    /// Command to execute: get_device, get_herd, post_event
    #[arg(short, long)]
    command: String,

    /// Scout URL
    #[arg(long, name = "scout_url", default_value = "http://localhost:3000/api/scout")]
    scout_url: String,

    /// API Key (or set SCOUT_DEVICE_API_KEY env var)
    #[arg(long, name = "api_key")]
    api_key: Option<String>,

    /// Herd ID (for get_herd command)
    #[arg(long, name = "herd_id")]
    herd_id: Option<u32>,

    /// Event data as JSON (for post_event command)
    #[arg(long, name = "event_json")]
    event_json: Option<String>,

    /// Tags data as JSON array (for post_event command)
    #[arg(long, name = "tags_json")]
    tags_json: Option<String>,

    /// File path (for post_event command)
    #[arg(long, name = "file_path")]
    file_path: Option<String>,
}

// example usage:
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command get_device
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command get_herd
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command post_event --event_json '{"message": "Test event", "media_url": "https://example.com/image.jpg", "file_path": "path/to/image.jpg", "location": "Point(0,0)", "altitude": 20.3, "heading": 90.0, "media_type": "image", "device_id": "123", "earthranger_url": null, "timestamp_observation": "2024-01-01T00:00:00Z", "is_public": true, "session_id": null}' --tags_json '[]' --file_path 'path/to/image.jpg'

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Get API key from args or environment
    let api_key = args.api_key.unwrap_or_else(|| {
        env::var("SCOUT_DEVICE_API_KEY").expect("SCOUT_DEVICE_API_KEY environment variable not set")
    });

    let mut client = ScoutClient::new(args.scout_url, api_key)?;

    match args.command.as_str() {
        "get_device" => {
            let response = client.get_device().await?;
            if response.status == ResponseScoutStatus::Success {
                if let Some(device) = response.data {
                    println!("{}", serde_json::to_string_pretty(&device)?);
                } else {
                    println!("{{}}");
                }
            } else {
                eprintln!("Failed to get device: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "get_herd" => {
            let response = client.get_herd(args.herd_id).await?;
            if response.status == ResponseScoutStatus::Success {
                if let Some(herd) = response.data {
                    println!("{}", serde_json::to_string_pretty(&herd)?);
                } else {
                    println!("{{}}");
                }
            } else {
                eprintln!("Failed to get herd: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "post_event" => {
            let event_json = args.event_json.expect("event_json required for post_event");
            let tags_json = args.tags_json.expect("tags_json required for post_event");
            let file_path = args.file_path.expect("file_path required for post_event");

            // Parse event JSON
            let event: Event = serde_json::from_str(&event_json)?;

            // Parse tags JSON
            let tags: Vec<Tag> = serde_json::from_str(&tags_json)?;

            let response = client.post_event_with_tags(&event, &tags, &file_path).await?;
            if response.status == ResponseScoutStatus::Success {
                println!("Event posted successfully");
            } else {
                eprintln!("Failed to post event: {:?}", response.status);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args.command);
            eprintln!("Available commands: get_device, get_herd, post_event");
            std::process::exit(1);
        }
    }

    Ok(())
}
