use clap::Parser;
use serde_json;
use std::env;
use scout_rs::client::{ ScoutClient, Event, Tag, Plan, ResponseScoutStatus };

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, rename_all = "snake_case")]
struct Args {
    /// Command to execute: get_device, get_herd, get_plans_by_herd, get_plan_by_id, create_plan, update_plan, delete_plan, post_event, update_event, delete_event
    #[arg(short, long)]
    command: String,

    /// Scout URL
    #[arg(long, name = "scout_url", default_value = "http://localhost:3000/api/scout")]
    scout_url: String,

    /// API Key (or set SCOUT_DEVICE_API_KEY env var)
    #[arg(long, name = "api_key")]
    api_key: Option<String>,

    /// Herd ID (required for plan operations, optional for get_herd command)
    #[arg(long, name = "herd_id")]
    herd_id: Option<i64>,

    /// Event data as JSON (for post_event and update_event commands)
    #[arg(long, name = "event_json")]
    event_json: Option<String>,

    /// Tags data as JSON array (for post_event command)
    #[arg(long, name = "tags_json")]
    tags_json: Option<String>,
    /// Location latitude for tags (optional)
    #[arg(long, name = "tag_latitude")]
    tag_latitude: Option<f64>,

    /// Location longitude for tags (optional)
    #[arg(long, name = "tag_longitude")]
    tag_longitude: Option<f64>,
    /// File path (for post_event command)
    #[arg(long, name = "file_path")]
    file_path: Option<String>,

    /// Event ID (for update_event and delete_event commands)
    #[arg(long, name = "event_id")]
    event_id: Option<i64>,

    /// Plan ID (for get_plan_by_id, update_plan, and delete_plan commands)
    #[arg(long, name = "plan_id")]
    plan_id: Option<i64>,

    /// Plan data as JSON (for create_plan and update_plan commands)
    #[arg(long, name = "plan_json")]
    plan_json: Option<String>,
}

// example usage:
// With location support for tags:
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command post_event --event_json '{"message": "Test event", "media_url": "https://example.com/image.jpg", "file_path": "path/to/image.jpg", "location": "POINT(0,0)", "altitude": 20.3, "heading": 90.0, "media_type": "image", "device_id": "123", "earthranger_url": null, "timestamp_observation": "2024-01-01T00:00:00Z", "is_public": true, "session_id": null}' --tags_json '[{"x": 0.5, "y": 0.5, "width": 0.2, "height": 0.2, "conf": 0.9, "observation_type": "manual", "class_name": "animal", "event_id": 0}]' --file_path 'path/to/image.jpg' --tag_latitude 40.7128 --tag_longitude -74.0060
// This will create tags with location coordinates extracted from the provided latitude/longitude// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command get_device
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command get_herd
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command get_plans_by_herd --herd_id 123
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command get_plan_by_id --plan_id 123 --herd_id 123
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command create_plan --plan_json '{"name": "Test Plan", "instructions": "Test instructions", "herd_id": 123, "plan_type": "mission"}' --herd_id 123
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command update_plan --plan_id 123 --plan_json '{"name": "Updated Plan", "instructions": "Updated instructions", "herd_id": 123, "plan_type": "fence"}' --herd_id 123
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command delete_plan --plan_id 123 --herd_id 123
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command post_event --event_json '{"message": "Test event", "media_url": "https://example.com/image.jpg", "file_path": "path/to/image.jpg", "location": "POINT(0,0)", "altitude": 20.3, "heading": 90.0, "media_type": "image", "device_id": "123", "earthranger_url": null, "timestamp_observation": "2024-01-01T00:00:00Z", "is_public": true, "session_id": null}' --tags_json '[{"x": 0.5, "y": 0.5, "width": 0.2, "height": 0.2, "conf": 0.9, "observation_type": "manual", "class_name": "animal", "event_id": 0}]' --file_path 'path/to/image.jpg' --tag_latitude 40.7128 --tag_longitude -74.0060
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command update_event --event_id 123 --event_json '{"message": "Updated event", "media_url": "https://example.com/updated.jpg", "file_path": "path/to/image.jpg", "location": "POINT(0,0)", "altitude": 25.0, "heading": 180.0, "media_type": "image", "device_id": "123", "earthranger_url": null, "timestamp_observation": "2024-01-01T00:00:00Z", "is_public": false, "session_id": null, "id": 123}'
// SCOUT_DEVICE_API_KEY=1234567890 ./target/release/scout_cli --command delete_event --event_id 123

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Get API key from args or environment
    let api_key = args.api_key.unwrap_or_else(|| {
        env::var("SCOUT_DEVICE_API_KEY").expect("SCOUT_DEVICE_API_KEY environment variable not set")
    });

    let mut client = ScoutClient::new(api_key)?;

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
        "get_plans_by_herd" => {
            let herd_id = args.herd_id.expect("herd_id required for get_plans_by_herd");

            // Identify the client first to establish database connection
            if let Err(e) = client.identify().await {
                eprintln!("Failed to identify client: {}", e);
                std::process::exit(1);
            }

            let response = client.get_plans_by_herd(herd_id).await?;
            if response.status == ResponseScoutStatus::Success {
                if let Some(plans) = response.data {
                    println!("{}", serde_json::to_string_pretty(&plans)?);
                } else {
                    println!("[]");
                }
            } else {
                eprintln!("Failed to get plans: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "get_plan_by_id" => {
            let plan_id = args.plan_id.expect("plan_id required for get_plan_by_id");
            let herd_id = args.herd_id.expect("herd_id required for get_plan_by_id");

            // Identify the client first to establish database connection
            if let Err(e) = client.identify().await {
                eprintln!("Failed to identify client: {}", e);
                std::process::exit(1);
            }

            // Verify the plan belongs to the specified herd
            let response = client.get_plan_by_id(plan_id).await?;
            if response.status == ResponseScoutStatus::Success {
                if let Some(plan) = response.data {
                    if plan.herd_id != herd_id {
                        eprintln!("Plan {} does not belong to herd {}", plan_id, herd_id);
                        std::process::exit(1);
                    }
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                } else {
                    println!("{{}}");
                }
            } else {
                eprintln!("Failed to get plan: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "create_plan" => {
            let plan_json = args.plan_json.expect("plan_json required for create_plan");
            let herd_id = args.herd_id.expect("herd_id required for create_plan");

            // Identify the client first to establish database connection
            if let Err(e) = client.identify().await {
                eprintln!("Failed to identify client: {}", e);
                std::process::exit(1);
            }

            // Parse plan JSON
            let mut plan: Plan = serde_json::from_str(&plan_json)?;

            // Ensure the herd_id matches
            plan.herd_id = herd_id;

            let response = client.create_plan(&plan).await?;
            if response.status == ResponseScoutStatus::Success {
                if let Some(created_plan) = response.data {
                    println!("Plan created successfully");
                    println!("{}", serde_json::to_string_pretty(&created_plan)?);
                } else {
                    println!("Plan created successfully (no data returned)");
                }
            } else {
                eprintln!("Failed to create plan: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "update_plan" => {
            let plan_id = args.plan_id.expect("plan_id required for update_plan");
            let plan_json = args.plan_json.expect("plan_json required for update_plan");
            let herd_id = args.herd_id.expect("herd_id required for update_plan");

            // Identify the client first to establish database connection
            if let Err(e) = client.identify().await {
                eprintln!("Failed to identify client: {}", e);
                std::process::exit(1);
            }

            // Parse plan JSON
            let mut plan: Plan = serde_json::from_str(&plan_json)?;

            // Ensure the herd_id matches
            plan.herd_id = herd_id;

            let response = client.update_plan(plan_id, &plan).await?;
            if response.status == ResponseScoutStatus::Success {
                if let Some(updated_plan) = response.data {
                    println!("Plan updated successfully");
                    println!("{}", serde_json::to_string_pretty(&updated_plan)?);
                } else {
                    println!("Plan updated successfully (no data returned)");
                }
            } else {
                eprintln!("Failed to update plan: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "delete_plan" => {
            let plan_id = args.plan_id.expect("plan_id required for delete_plan");
            let herd_id = args.herd_id.expect("herd_id required for delete_plan");

            // Identify the client first to establish database connection
            if let Err(e) = client.identify().await {
                eprintln!("Failed to identify client: {}", e);
                std::process::exit(1);
            }

            // First verify the plan belongs to the specified herd before deleting
            let plan_response = client.get_plan_by_id(plan_id).await?;
            if plan_response.status == ResponseScoutStatus::Success {
                if let Some(plan) = plan_response.data {
                    if plan.herd_id != herd_id {
                        eprintln!(
                            "Failed to delete plan: Plan {} does not belong to herd {}",
                            plan_id,
                            herd_id
                        );
                        std::process::exit(1);
                    }
                }
            }

            let response = client.delete_plan(plan_id).await?;
            if response.status == ResponseScoutStatus::Success {
                println!("Plan deleted successfully");
            } else {
                eprintln!("Failed to delete plan: {:?}", response.status);
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

            // Apply location to tags if provided
            let mut tags: Vec<Tag> = serde_json::from_str(&tags_json)?;
            if let (Some(lat), Some(lon)) = (args.tag_latitude, args.tag_longitude) {
                for tag in &mut tags {
                    tag.set_location(lat, lon);
                }
            }
            let response = client.create_event_with_tags(&event, &tags, Some(&file_path)).await?;
            if response.status == ResponseScoutStatus::Success {
                println!("Event posted successfully");
            } else {
                eprintln!("Failed to post event: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "update_event" => {
            let event_id = args.event_id.expect("event_id required for update_event");
            let event_json = args.event_json.expect("event_json required for update_event");

            // Parse event JSON
            let event: Event = serde_json::from_str(&event_json)?;

            let response = client.update_event(event_id, &event).await?;
            if response.status == ResponseScoutStatus::Success {
                if let Some(updated_event) = response.data {
                    println!("Event updated successfully");
                    println!("{}", serde_json::to_string_pretty(&updated_event)?);
                } else {
                    println!("Event updated successfully (no data returned)");
                }
            } else {
                eprintln!("Failed to update event: {:?}", response.status);
                std::process::exit(1);
            }
        }
        "delete_event" => {
            let event_id = args.event_id.expect("event_id required for delete_event");

            let response = client.delete_event(event_id).await?;
            if response.status == ResponseScoutStatus::Success {
                println!("Event deleted successfully");
            } else {
                eprintln!("Failed to delete event: {:?}", response.status);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args.command);
            eprintln!(
                "Available commands: get_device, get_herd, get_plans_by_herd, get_plan_by_id, create_plan, update_plan, delete_plan, post_event, update_event, delete_event"
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
