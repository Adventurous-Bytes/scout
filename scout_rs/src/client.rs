use anyhow::{anyhow, Result};

use crate::db_client::{DatabaseConfig, ScoutDbClient};
use crate::models::*;

// ===== CLIENT IMPLEMENTATION =====

#[derive(Debug)]
pub struct ScoutClient {
    pub api_key: String,
    pub device: Option<Device>,
    pub herd: Option<Herd>,
    db_client: Option<ScoutDbClient>,
}

impl ScoutClient {
    /// Creates a new ScoutClient instance.
    ///
    /// # Arguments
    /// * `api_key` - The API key for the Scout service.
    pub fn new(api_key: String) -> Result<Self> {
        Ok(Self {
            api_key,
            device: None,
            herd: None,
            db_client: None,
        })
    }

    /// Identifies the device and herd, then establishes direct database connection
    pub async fn identify(&mut self) -> Result<()> {
        let db_config = DatabaseConfig::from_env_with_api_key(Some(self.api_key.clone()))?;
        let mut db_client = ScoutDbClient::new(db_config);
        db_client.connect()?;

        self.db_client = Some(db_client);

        let device = self.get_device_from_db().await?;

        let herd = self.get_herd_from_db(device.herd_id).await?;

        self.device = Some(device);
        self.herd = Some(herd);

        Ok(())
    }

    /// Gets device information using get_device_by_api_key function and parsing JSON response
    async fn get_device_from_db(&mut self) -> Result<Device> {
        let api_key = self.api_key.clone();
        let db_client = self.get_db_client()?;

        // Call get_device_by_api_key function
        let client = db_client.get_client()?;
        let response = client
            .rpc(
                "get_device_by_api_key",
                serde_json::json!({
                    "device_api_key": api_key
                })
                .to_string(),
            )
            .execute()
            .await?;

        let body = response.text().await?;

        // Parse the JSON response as DevicePrettyLocation
        let device_pretty: DevicePrettyLocation = serde_json::from_str(&body).map_err(|e| {
            anyhow!(
                "Failed to parse device response: {} - Response: {}",
                e,
                body
            )
        })?;

        // Convert DevicePrettyLocation to Device
        let device = Device {
            id: device_pretty.id,
            id_local: None,
            inserted_at: device_pretty.inserted_at,
            created_by: device_pretty.created_by,
            herd_id: device_pretty.herd_id,
            device_type: DeviceType::from(device_pretty.device_type.as_str()),
            name: device_pretty.name,
            description: device_pretty.description,
            domain_name: device_pretty.domain_name,
            altitude: device_pretty.altitude,
            heading: device_pretty.heading,
            location: device_pretty.location,
            video_publisher_token: None,
            video_subscriber_token: None,
        };

        Ok(device)
    }

    /// Gets herd information directly from database
    async fn get_herd_from_db(&mut self, herd_id: i64) -> Result<Herd> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("herds")
                    .select("*")
                    .eq("id", herd_id.to_string())
                    .limit(1)
            })
            .await?;

        if results.is_empty() {
            return Err(anyhow!("No herd found for ID: {}", herd_id));
        }

        Ok(results.into_iter().next().unwrap())
    }

    /// Gets the database client, ensuring it's available
    fn get_db_client(&mut self) -> Result<&mut ScoutDbClient> {
        self.db_client
            .as_mut()
            .ok_or_else(|| anyhow!("Database client not initialized. Call identify() first."))
    }

    /// Checks if the client has been identified and has a database connection
    pub fn is_identified(&self) -> bool {
        self.db_client.is_some() && self.device.is_some() && self.herd.is_some()
    }

    // ===== HELPER METHODS =====

    /// Checks if a session exists in the database by device_id, start timestamp, and end timestamp
    pub async fn does_session_exist(
        &mut self,
        device_id: i64,
        timestamp_start: &str,
    ) -> Result<bool> {
        let db_client = self.get_db_client()?;

        #[derive(Debug, serde::Deserialize)]
        struct SessionIdOnly {
            id: i64,
        }

        let results: Vec<SessionIdOnly> = db_client
            .query(|client| {
                client
                    .from("sessions")
                    .select("id")
                    .eq("device_id", device_id.to_string())
                    .eq("timestamp_start", timestamp_start)
                    .limit(1)
            })
            .await?;

        Ok(!results.is_empty())
    }

    /// Convenience method to check if a session exists using a Session object
    pub async fn does_session_exist_from_session(&mut self, session: &Session) -> Result<bool> {
        self.does_session_exist(session.device_id, &session.timestamp_start)
            .await
    }

    /// Helper to create a success response
    fn success_response<T>(data: T) -> ResponseScout<T> {
        ResponseScout::new(ResponseScoutStatus::Success, Some(data))
    }

    /// Helper to create a failure response
    fn failure_response<T>() -> ResponseScout<T> {
        ResponseScout::new(ResponseScoutStatus::Failure, None)
    }

    /// Helper to handle database insert results
    fn handle_insert_result<T>(result: Vec<T>) -> Result<ResponseScout<T>> {
        if result.is_empty() {
            Ok(Self::failure_response())
        } else {
            Ok(Self::success_response(result.into_iter().next().unwrap()))
        }
    }

    /// Helper to handle database query results
    fn handle_query_result<T>(result: Vec<T>) -> ResponseScout<Vec<T>> {
        Self::success_response(result)
    }

    // ===== BACKWARD COMPATIBILITY METHODS =====

    /// Gets device information (backward compatibility method)
    pub async fn get_device(&mut self) -> Result<ResponseScout<Device>> {
        if let Some(device) = &self.device {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(device.clone()),
            ));
        }

        self.identify().await?;

        if let Some(device) = &self.device {
            Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(device.clone()),
            ))
        } else {
            Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
        }
    }

    /// Gets herd information (backward compatibility method)
    pub async fn get_herd(&mut self, herd_id: Option<i64>) -> Result<ResponseScout<Herd>> {
        let herd_id = if let Some(id) = herd_id {
            id
        } else if let Some(device) = &self.device {
            device.herd_id
        } else {
            return Err(anyhow!("No herd_id provided and no device data available"));
        };

        if let Some(herd) = &self.herd {
            if herd.id == Some(herd_id) {
                return Ok(ResponseScout::new(
                    ResponseScoutStatus::Success,
                    Some(herd.clone()),
                ));
            }
        }

        if self.device.is_none() {
            self.identify().await?;
        }

        if let Some(herd) = &self.herd {
            if herd.id == Some(herd_id) {
                Ok(ResponseScout::new(
                    ResponseScoutStatus::Success,
                    Some(herd.clone()),
                ))
            } else {
                Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
            }
        } else {
            Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
        }
    }

    // ===== DIRECT DATABASE OPERATIONS =====

    /// Creates an event directly in the database
    pub async fn create_event(&mut self, event: &Event) -> Result<ResponseScout<Event>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("events", event).await?;
        Self::handle_insert_result(result)
    }

    /// Creates tags for an event directly in the database
    /// RLS policies and foreign key constraints handle validation automatically
    pub async fn create_tags(
        &mut self,
        event_id: i64,
        tags: &[Tag],
    ) -> Result<ResponseScout<Vec<Tag>>> {
        let db_client = self.get_db_client()?;

        if tags.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        // Prepare tags with event_id for bulk insert
        let tags_with_event_id: Vec<Tag> = tags
            .iter()
            .map(|tag| {
                let mut tag_with_event_id = tag.clone();
                tag_with_event_id.update_event_id(event_id);
                tag_with_event_id
            })
            .collect();

        // Use bulk insert for better performance
        let result = db_client.insert_bulk("tags", &tags_with_event_id).await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Creates an event with tags (compatibility method)
    pub async fn create_event_with_tags(
        &mut self,
        event: &Event,
        tags: &[Tag],
        _file_path: Option<&str>,
    ) -> Result<ResponseScout<Event>> {
        let event_response = self.create_event(event).await?;

        if event_response.status != ResponseScoutStatus::Success {
            return Ok(event_response);
        }

        let created_event = event_response.data.unwrap();

        if !tags.is_empty() {
            let tags_response = self.create_tags(created_event.id.unwrap(), tags).await?;
            if tags_response.status != ResponseScoutStatus::Success {
                return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
            }
        }

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(created_event),
        ))
    }

    /// Creates a session directly in the database
    pub async fn create_session(&mut self, session: &Session) -> Result<ResponseScout<Session>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("sessions", session).await?;
        Self::handle_insert_result(result)
    }

    /// Creates connectivity data directly from the database
    pub async fn create_connectivity(
        &mut self,
        connectivity: &Connectivity,
    ) -> Result<ResponseScout<Connectivity>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("connectivity", connectivity).await?;
        Self::handle_insert_result(result)
    }

    /// Gets sessions for a herd directly from the database
    pub async fn get_sessions_by_herd(
        &mut self,
        herd_id: i64,
    ) -> Result<ResponseScout<Vec<Session>>> {
        let db_client = self.get_db_client()?;
        let results = db_client
            .query(|client| {
                client
                    .from("sessions")
                    .select("*, devices!inner(herd_id)")
                    .eq("devices.herd_id", herd_id.to_string())
                    .order("timestamp_start.desc")
            })
            .await?;
        Ok(Self::handle_query_result(results))
    }

    /// Gets plans for a herd directly from the database
    pub async fn get_plans_by_herd(&mut self, herd_id: i64) -> Result<ResponseScout<Vec<Plan>>> {
        let db_client = self.get_db_client()?;
        let results = db_client
            .query(|client| {
                client
                    .from("plans")
                    .eq("herd_id", herd_id.to_string())
                    .order("inserted_at.desc")
            })
            .await?;

        // Return empty results if no plans found (don't panic)
        Ok(Self::handle_query_result(results))
    }

    /// Gets a specific plan by ID directly from the database
    pub async fn get_plan_by_id(&mut self, plan_id: i64) -> Result<ResponseScout<Plan>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("plans")
                    .select("*")
                    .eq("id", plan_id.to_string())
                    .limit(1)
            })
            .await?;

        // Return failure status if no plan found (don't panic)
        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        if results.len() > 1 {
            panic!(
                "Multiple plans found with ID: {}. Expected exactly one plan.",
                plan_id
            );
        }

        let plan = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(plan)))
    }

    /// Creates a plan directly in the database
    pub async fn create_plan(&mut self, plan: &Plan) -> Result<ResponseScout<Plan>> {
        let db_client = self.get_db_client()?;

        // Create a plan for insertion without ID field
        let plan_for_insert = PlanInsert {
            id: None,          // Will be auto-generated by database
            id_local: None,    // Add if you are upserting to sync engine
            inserted_at: None, // Database will use default value
            name: plan.name.clone(),
            instructions: plan.instructions.clone(),
            herd_id: plan.herd_id,
            plan_type: plan.plan_type.clone(),
        };

        let result = db_client.insert("plans", &plan_for_insert).await?;

        // Convert PlanInsert results back to Plan with generated IDs
        let plans: Vec<Plan> = result
            .into_iter()
            .map(|p| Plan {
                id: Some(p.id.unwrap_or(0)), // Use generated ID or fallback to 0
                id_local: None,
                inserted_at: p.inserted_at,
                name: p.name,
                instructions: p.instructions,
                herd_id: p.herd_id,
                plan_type: p.plan_type,
            })
            .collect();

        Self::handle_insert_result(plans)
    }

    /// Updates a plan directly in the database
    pub async fn update_plan(&mut self, plan_id: i64, plan: &Plan) -> Result<ResponseScout<Plan>> {
        let db_client = self.get_db_client()?;

        let result = db_client
            .update(plan, |client| {
                client.from("plans").eq("id", plan_id.to_string())
            })
            .await?;

        if result.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let updated_plan = result.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(updated_plan),
        ))
    }

    /// Deletes a plan directly from the database
    pub async fn delete_plan(&mut self, plan_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        db_client
            .delete(|client| client.from("plans").eq("id", plan_id.to_string()))
            .await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    /// Gets events for a session directly from the database
    pub async fn get_session_events(
        &mut self,
        session_id: i64,
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;
        let results = db_client
            .query(|client| {
                client
                    .from("events")
                    .eq("session_id", session_id.to_string())
                    .order("timestamp_observation.desc")
            })
            .await?;
        Ok(Self::handle_query_result(results))
    }

    /// Gets connectivity data for a session directly from the database
    pub async fn get_session_connectivity(
        &mut self,
        session_id: i64,
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        let db_client = self.get_db_client()?;
        let results = db_client
            .query(|client| {
                client
                    .from("connectivity")
                    .eq("session_id", session_id.to_string())
                    .order("timestamp_start.asc")
            })
            .await?;
        Ok(Self::handle_query_result(results))
    }

    /// Updates a session directly in the database
    pub async fn update_session(
        &mut self,
        session_id: i64,
        session: &Session,
    ) -> Result<ResponseScout<Session>> {
        let db_client = self.get_db_client()?;

        let result = db_client
            .update(session, |client| {
                client.from("sessions").eq("id", session_id.to_string())
            })
            .await?;

        if result.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let updated_session = result.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(updated_session),
        ))
    }

    /// Deletes a session directly from the database
    /// Database cascade deletion handles dependent records automatically
    pub async fn delete_session(&mut self, session_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        let session_deleted = db_client
            .delete(|client| client.from("sessions").eq("id", session_id.to_string()))
            .await;

        if session_deleted.is_err() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    /// Deletes an event directly from the database
    /// Database cascade deletion handles dependent records automatically
    pub async fn delete_event(&mut self, event_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        let _event_deleted = db_client
            .delete(|client| client.from("events").eq("id", event_id.to_string()))
            .await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    /// Deletes a tag directly from the database
    pub async fn delete_tag(&mut self, tag_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        db_client
            .delete(|client| client.from("tags").eq("id", tag_id.to_string()))
            .await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    /// Deletes connectivity data directly from the database
    pub async fn delete_connectivity(&mut self, connectivity_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        db_client
            .delete(|client| {
                client
                    .from("connectivity")
                    .eq("id", connectivity_id.to_string())
            })
            .await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    // ===== ADDITIONAL OPERATIONS =====

    /// Gets all devices for a herd directly from the database
    pub async fn get_devices_by_herd(
        &mut self,
        herd_id: i64,
    ) -> Result<ResponseScout<Vec<Device>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("devices")
                    .eq("herd_id", herd_id.to_string())
                    .order("inserted_at.desc")
            })
            .await?;

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(results),
        ))
    }

    /// Gets a specific event by ID directly from the database
    pub async fn get_event_by_id(&mut self, event_id: i64) -> Result<ResponseScout<Event>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("events")
                    .select("*")
                    .eq("id", event_id.to_string())
                    .limit(1)
            })
            .await?;

        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let event = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(event),
        ))
    }

    /// Gets a specific device by ID directly from the database
    pub async fn get_device_by_id(&mut self, device_id: i64) -> Result<ResponseScout<Device>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("devices")
                    .select("*")
                    .eq("id", device_id.to_string())
                    .limit(1)
            })
            .await?;

        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let device = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(device),
        ))
    }

    /// Gets a specific herd by ID directly from the database
    pub async fn get_herd_by_id(&mut self, herd_id: i64) -> Result<ResponseScout<Herd>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("herds")
                    .select("*")
                    .eq("id", herd_id.to_string())
                    .limit(1)
            })
            .await?;

        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let device = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(device),
        ))
    }

    /// Gets all events for a device directly from the database
    pub async fn get_device_events(&mut self, device_id: i64) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("events")
                    .eq("device_id", device_id.to_string())
                    .order("timestamp_observation.desc")
            })
            .await?;

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(results),
        ))
    }

    /// Gets events with tags for a device directly from the database
    pub async fn get_device_events_with_tags(
        &mut self,
        device_id: i64,
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("events")
                    .select("*, tags(*)")
                    .eq("device_id", device_id.to_string())
                    .order("timestamp_observation.desc")
            })
            .await?;

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(results),
        ))
    }

    /// Gets events with tags for a device using the database function
    pub async fn get_device_events_with_tags_via_function(
        &mut self,
        device_id: i64,
        limit: i64,
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client.rpc(
                    "get_events_and_tags_for_device",
                    serde_json::json!({
                        "device_id_caller": device_id,
                        "limit_caller": limit
                    })
                    .to_string(),
                )
            })
            .await?;

        Ok(Self::handle_query_result(results))
    }

    /// Gets events within a time range directly from the database
    pub async fn get_events_in_timerange(
        &mut self,
        start_time: &str,
        end_time: &str,
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("events")
                    .gte("timestamp_observation", start_time)
                    .lte("timestamp_observation", end_time)
                    .order("timestamp_observation.desc")
            })
            .await?;

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(results),
        ))
    }

    /// Gets events within a geographic area directly from the database
    pub async fn get_events_in_area(
        &mut self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64,
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("events")
                    .select("*")
                    .gte("latitude", min_lat.to_string())
                    .lte("latitude", max_lat.to_string())
                    .gte("longitude", min_lon.to_string())
                    .lte("longitude", max_lon.to_string())
                    .order("timestamp_observation.desc")
            })
            .await?;

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(results),
        ))
    }

    /// Creates multiple events in a batch directly in the database
    pub async fn create_events_batch(
        &mut self,
        events: &[Event],
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        if events.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        // Use bulk insert for better performance
        let result = db_client.insert_bulk("events", events).await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Creates multiple sessions in a batch directly in the database
    pub async fn create_sessions_batch(
        &mut self,
        sessions: &[Session],
    ) -> Result<ResponseScout<Vec<Session>>> {
        let db_client = self.get_db_client()?;

        if sessions.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        // Use bulk insert for better performance
        let result = db_client.insert_bulk("sessions", sessions).await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Creates multiple connectivity entries in a batch directly in the database
    pub async fn create_connectivity_batch(
        &mut self,
        connectivity_entries: &[Connectivity],
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        let db_client = self.get_db_client()?;

        if connectivity_entries.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        // Use bulk insert for better performance
        let result = db_client
            .insert_bulk("connectivity", connectivity_entries)
            .await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Upserts multiple sessions in a batch (insert or update on conflict)
    pub async fn upsert_sessions_batch(
        &mut self,
        sessions: &[Session],
    ) -> Result<ResponseScout<Vec<Session>>> {
        let db_client = self.get_db_client()?;

        if sessions.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        let result = db_client.upsert_bulk("sessions", sessions).await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Upserts multiple connectivity entries in a batch (insert or update on conflict)
    pub async fn upsert_connectivity_batch(
        &mut self,
        connectivity_entries: &[Connectivity],
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        let db_client = self.get_db_client()?;

        if connectivity_entries.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        let result = db_client
            .upsert_bulk("connectivity", connectivity_entries)
            .await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Upserts multiple events in a batch (insert or update on conflict)
    pub async fn upsert_events_batch(
        &mut self,
        events: &[Event],
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        if events.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        let result = db_client.upsert_bulk("events", events).await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Upserts multiple tags in a batch (insert or update on conflict)
    pub async fn upsert_tags_batch(&mut self, tags: &[Tag]) -> Result<ResponseScout<Vec<Tag>>> {
        let db_client = self.get_db_client()?;

        if tags.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        let result = db_client.upsert_bulk("tags", tags).await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Updates an event directly in the database
    pub async fn update_event(
        &mut self,
        event_id: i64,
        event: &Event,
    ) -> Result<ResponseScout<Event>> {
        let db_client = self.get_db_client()?;

        let result = db_client
            .update(event, |client| {
                client.from("events").eq("id", event_id.to_string())
            })
            .await?;

        if result.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let updated_event = result.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(updated_event),
        ))
    }

    /// Updates connectivity data directly in the database
    pub async fn update_connectivity(
        &mut self,
        connectivity_id: i64,
        connectivity: &Connectivity,
    ) -> Result<ResponseScout<Connectivity>> {
        let db_client = self.get_db_client()?;

        let result = db_client
            .update(connectivity, |client| {
                client
                    .from("connectivity")
                    .eq("id", connectivity_id.to_string())
            })
            .await?;

        if result.is_empty() {
            return Err(anyhow!(
                "Failed to update connectivity entry - no data returned"
            ));
        }

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result.into_iter().next().unwrap()),
        ))
    }

    /// Gets connectivity data with coordinates directly from the database
    pub async fn get_connectivity_with_coordinates(
        &mut self,
        session_id: i64,
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("connectivity")
                    .eq("session_id", session_id.to_string())
                    .order("timestamp_start.asc")
            })
            .await?;

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(results),
        ))
    }

    /// Ends a session by updating its timestamp_end directly in the database
    pub async fn end_session(
        &mut self,
        session_id: i64,
        timestamp_end: u64,
    ) -> Result<ResponseScout<()>> {
        let mut session = Session::new(
            0,
            timestamp_end,
            Some(timestamp_end),
            "".to_string(),
            None,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        session.timestamp_end = Some(
            chrono::DateTime::from_timestamp(timestamp_end as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339(),
        );

        let response = self.update_session(session_id, &session).await?;
        if response.status == ResponseScoutStatus::Success {
            Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
        } else {
            Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
        }
    }

    /// Gets statistics for a session directly from the database
    pub async fn get_session_statistics(
        &mut self,
        session_id: i64,
    ) -> Result<ResponseScout<Session>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("sessions")
                    .select("*")
                    .eq("id", session_id.to_string())
                    .limit(1)
            })
            .await?;

        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let session = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(session),
        ))
    }

    // ===== COMPATIBILITY METHODS =====

    /// Compatibility method for post_events_batch
    pub async fn post_events_batch(
        &mut self,
        events_and_files: &[(Event, Vec<Tag>, String)],
        _batch_size: usize,
    ) -> Result<ResponseScout<Vec<Event>>> {
        let mut created_events = Vec::new();

        for (event, tags, _file_path) in events_and_files {
            let event_response = self.create_event(event).await?;
            if event_response.status != ResponseScoutStatus::Success {
                return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
            }

            let created_event = event_response.data.unwrap();
            if !tags.is_empty() {
                let tags_response = self.create_tags(created_event.id.unwrap(), tags).await?;
                if tags_response.status != ResponseScoutStatus::Success {
                    return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
                }
            }
            created_events.push(created_event);
        }

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(created_events),
        ))
    }

    /// Gets zones and actions for a herd directly from the database
    pub async fn get_zones_and_actions_by_herd(
        &mut self,
        herd_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<ResponseScout<Vec<Zone>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("zones_and_actions")
                    .eq("herd_id", herd_id.to_string())
                    .order("inserted_at.desc")
                    .range(offset as usize, (offset + limit - 1) as usize)
            })
            .await?;

        Ok(Self::handle_query_result(results))
    }

    // ===== ARTIFACT OPERATIONS =====

    /// Creates an artifact directly in the database
    pub async fn create_artifact(
        &mut self,
        artifact: &Artifact,
    ) -> Result<ResponseScout<Artifact>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("artifacts", artifact).await?;
        Self::handle_insert_result(result)
    }

    /// Gets artifacts for a session directly from the database
    pub async fn get_artifacts_by_session(
        &mut self,
        session_id: i64,
    ) -> Result<ResponseScout<Vec<Artifact>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("artifacts")
                    .eq("session_id", session_id.to_string())
                    .order("created_at.desc")
            })
            .await?;

        Ok(Self::handle_query_result(results))
    }

    /// Gets all artifacts for a herd (via sessions) directly from the database
    pub async fn get_artifacts_by_herd(
        &mut self,
        herd_id: i64,
    ) -> Result<ResponseScout<Vec<Artifact>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("artifacts")
                    .select("*, sessions!inner(device_id), devices!inner(herd_id)")
                    .eq("devices.herd_id", herd_id.to_string())
                    .order("created_at.desc")
            })
            .await?;

        Ok(Self::handle_query_result(results))
    }

    /// Updates an artifact directly in the database
    pub async fn update_artifact(
        &mut self,
        artifact_id: i64,
        artifact: &Artifact,
    ) -> Result<ResponseScout<Artifact>> {
        let db_client = self.get_db_client()?;

        let result = db_client
            .update(artifact, |client| {
                client.from("artifacts").eq("id", artifact_id.to_string())
            })
            .await?;

        if result.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let updated_artifact = result.into_iter().next().unwrap();
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(updated_artifact),
        ))
    }

    /// Deletes an artifact directly from the database
    pub async fn delete_artifact(&mut self, artifact_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        db_client
            .delete(|client| client.from("artifacts").eq("id", artifact_id.to_string()))
            .await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    /// Creates multiple artifacts in a batch directly in the database
    pub async fn create_artifacts_batch(
        &mut self,
        artifacts: &[Artifact],
    ) -> Result<ResponseScout<Vec<Artifact>>> {
        let db_client = self.get_db_client()?;

        if artifacts.is_empty() {
            return Ok(ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(Vec::new()),
            ));
        }

        // Use bulk insert for better performance
        let result = db_client.insert_bulk("artifacts", artifacts).await?;
        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(result),
        ))
    }

    /// Creates a heartbeat record for a device
    pub async fn create_heartbeat(
        &mut self,
        heartbeat: &Heartbeat,
    ) -> Result<ResponseScout<Heartbeat>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("heartbeats", heartbeat).await?;
        Self::handle_insert_result(result)
    }

    /// Gets all heartbeats for a specific device
    pub async fn get_heartbeats_by_device(
        &mut self,
        device_id: i64,
    ) -> Result<ResponseScout<Vec<Heartbeat>>> {
        let db_client = self.get_db_client()?;

        let results = db_client
            .query(|client| {
                client
                    .from("heartbeats")
                    .select("*")
                    .eq("device_id", device_id.to_string())
                    .order("timestamp.desc")
            })
            .await?;

        Ok(ResponseScout::new(
            ResponseScoutStatus::Success,
            Some(results),
        ))
    }

    /// Deletes a heartbeat record by ID
    ///
    /// **Note:** This method is primarily intended for testing and cleanup purposes.
    /// In production, heartbeats are typically append-only for audit trail purposes.
    pub async fn delete_heartbeat(&mut self, heartbeat_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        let _heartbeat_deleted = db_client
            .delete(|client| client.from("heartbeats").eq("id", heartbeat_id.to_string()))
            .await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }
}
