use anyhow::{anyhow, Result};
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub rest_url: String,
    pub scout_api_key: String,
    pub supabase_api_key: String,
}

impl DatabaseConfig {
    /// Creates a new database config from environment variables
    pub fn from_env() -> Result<Self> {
        Self::from_env_with_api_key(None)
    }

    /// Creates a new database config from environment variables with an optional Scout device API key
    pub fn from_env_with_api_key(scout_device_api_key: Option<String>) -> Result<Self> {
        dotenv::dotenv().ok();

        let mut rest_url = std::env::var("SCOUT_DATABASE_REST_URL")
            .map_err(|_| anyhow!("SCOUT_DATABASE_REST_URL environment variable is required"))?;

        // Ensure the URL has the correct PostgREST path
        if !rest_url.ends_with("/rest/v1") {
            if rest_url.ends_with("/") {
                rest_url.push_str("rest/v1");
            } else {
                rest_url.push_str("/rest/v1");
            }
        }

        // Use provided API key or fall back to environment variable
        let scout_api_key = scout_device_api_key.unwrap_or_else(|| {
            std::env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| {
                eprintln!(
                    "Warning: SCOUT_DEVICE_API_KEY environment variable not found, using empty string"
                );
                String::new()
            })
        });

        let supabase_api_key = std::env::var("SUPABASE_PUBLIC_API_KEY").map_err(|_| {
            anyhow!("SUPABASE_PUBLIC_API_KEY environment variable is required for Supabase access")
        })?;

        Ok(DatabaseConfig {
            rest_url,
            scout_api_key,
            supabase_api_key,
        })
    }

    /// Gets the PostgREST endpoint URL
    pub fn get_rest_url(&self) -> &str {
        &self.rest_url
    }

    /// Gets the Scout API key for custom authentication
    pub fn get_scout_api_key(&self) -> &str {
        &self.scout_api_key
    }

    /// Gets the Supabase API key for PostgREST access
    pub fn get_supabase_api_key(&self) -> &str {
        &self.supabase_api_key
    }
}

pub struct ScoutDbClient {
    config: DatabaseConfig,
    client: Option<Postgrest>,
}

impl std::fmt::Debug for ScoutDbClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScoutDbClient")
            .field("config", &self.config)
            .field(
                "client",
                if self.client.is_some() {
                    &"Connected"
                } else {
                    &"Disconnected"
                },
            )
            .finish()
    }
}

impl ScoutDbClient {
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            client: None,
        }
    }

    /// Establishes a connection to the database via PostgREST
    pub fn connect(&mut self) -> Result<()> {
        let rest_url = self.config.get_rest_url();

        let client = Postgrest::new(rest_url)
            .insert_header("apikey", self.config.get_supabase_api_key())
            .insert_header("api_key", &format!("{}", self.config.get_scout_api_key()));

        self.client = Some(client);

        Ok(())
    }

    /// Gets the PostgREST client, ensuring connection is established
    pub fn get_client(&mut self) -> Result<&Postgrest> {
        if self.client.is_none() {
            self.connect()?;
        }

        self.client
            .as_ref()
            .ok_or_else(|| anyhow!("No PostgREST client available"))
    }

    /// Closes the database connection
    pub fn disconnect(&mut self) {
        if self.client.is_some() {
            self.client = None;
        }
    }

    /// Executes a query and returns the results
    pub async fn query<T>(
        &mut self,
        query_builder: impl FnOnce(&Postgrest) -> postgrest::Builder,
    ) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let client = self.get_client()?;

        let builder = query_builder(client);
        let response = builder.execute().await?;

        let body = response.text().await?;

        // Try to parse as the expected type first
        if let Ok(results) = serde_json::from_str::<Vec<T>>(&body) {
            Ok(results)
        } else {
            // If that fails, try to parse as an error response
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(error_msg) = error_response.get("error") {
                    return Err(anyhow!("Database error: {}", error_msg));
                } else if let Some(message) = error_response.get("message") {
                    return Err(anyhow!("Database message: {}", message));
                } else {
                    return Err(anyhow!("Database returned unexpected format: {}", body));
                }
            } else {
                return Err(anyhow!(
                    "Failed to parse database response as JSON: {}",
                    body
                ));
            }
        }
    }

    /// Executes a query that returns a single row
    pub async fn query_one<T>(
        &mut self,
        query_builder: impl FnOnce(&Postgrest) -> postgrest::Builder,
    ) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let client = self.get_client()?;

        let builder = query_builder(client);
        let response = builder.execute().await?;

        let body = response.text().await?;
        let results: Vec<T> = serde_json::from_str(&body)?;

        if results.is_empty() {
            return Err(anyhow!("No results found"));
        }

        Ok(results.into_iter().next().unwrap())
    }

    /// Executes a query that doesn't return results (INSERT, UPDATE, DELETE)
    pub async fn execute(
        &mut self,
        query_builder: impl FnOnce(&Postgrest) -> postgrest::Builder,
    ) -> Result<()> {
        let client = self.get_client()?;

        let builder = query_builder(client);
        let response = builder.execute().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Operation failed: HTTP {} - {}",
                status,
                error_text
            ));
        }

        Ok(())
    }

    /// Inserts data into a table
    pub async fn insert<T>(&mut self, table: &str, data: &T) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de> + serde::Serialize,
    {
        let client = self.get_client()?;

        let json_data = serde_json::to_string(data)?;

        let response = client.from(table).insert(&json_data).execute().await?;

        let body = response.text().await?;

        // Try to parse as the expected type first
        if let Ok(results) = serde_json::from_str::<Vec<T>>(&body) {
            Ok(results)
        } else {
            // If that fails, try to parse as an error response
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(error_msg) = error_response.get("error") {
                    return Err(anyhow!("Database insert error: {}", error_msg));
                } else if let Some(message) = error_response.get("message") {
                    return Err(anyhow!("Database insert message: {}", message));
                } else {
                    return Err(anyhow!(
                        "Database insert returned unexpected format: {}",
                        body
                    ));
                }
            } else {
                return Err(anyhow!(
                    "Failed to parse database insert response as JSON: {}",
                    body
                ));
            }
        }
    }

    /// Inserts multiple items in a single bulk operation
    pub async fn insert_bulk<T>(&mut self, table: &str, data: &[T]) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de> + serde::Serialize,
    {
        let client = self.get_client()?;

        let json_data = serde_json::to_string(data)?;

        let response = client.from(table).insert(&json_data).execute().await?;

        let body = response.text().await?;

        // Try to parse as the expected type first
        if let Ok(results) = serde_json::from_str::<Vec<T>>(&body) {
            Ok(results)
        } else {
            // If that fails, try to parse as an error response
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(error_msg) = error_response.get("error") {
                    return Err(anyhow!("Database bulk insert error: {}", error_msg));
                } else if let Some(message) = error_response.get("message") {
                    return Err(anyhow!("Database bulk insert message: {}", message));
                } else {
                    return Err(anyhow!(
                        "Database bulk insert returned unexpected format: {}",
                        body
                    ));
                }
            } else {
                return Err(anyhow!(
                    "Failed to parse database bulk insert response as JSON: {}",
                    body
                ));
            }
        }
    }

    /// Updates data in a table
    pub async fn update<T>(
        &mut self,
        data: &T,
        filter_builder: impl FnOnce(&Postgrest) -> postgrest::Builder,
    ) -> Result<Vec<T>>
    where
        T: for<'de> serde::Deserialize<'de> + serde::Serialize,
    {
        let client = self.get_client()?;

        let json_data = serde_json::to_string(data)?;

        let builder = filter_builder(client);
        let response = builder.update(&json_data).execute().await?;

        let body = response.text().await?;
        let results: Vec<T> = serde_json::from_str(&body)?;

        Ok(results)
    }

    /// Deletes data from a table
    pub async fn delete(
        &mut self,
        filter_builder: impl FnOnce(&Postgrest) -> postgrest::Builder,
    ) -> Result<()> {
        let client = self.get_client()?;

        let builder = filter_builder(client);
        let response = builder.delete().execute().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!(
                "Delete operation failed: HTTP {} - {}",
                status,
                error_text
            ));
        }

        Ok(())
    }
}

impl Drop for ScoutDbClient {
    fn drop(&mut self) {
        self.disconnect();
    }
}
