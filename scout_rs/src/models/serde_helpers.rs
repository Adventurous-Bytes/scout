// Serde helpers for model fields that may come from the API in multiple formats.

use serde::{Deserialize, Deserializer};

/// Deserializes an optional embedding from either a JSON array of floats or a pgvector-style string "[0.1, 0.2, ...]".
pub fn deserialize_embedding<'de, D>(deserializer: D) -> Result<Option<Vec<f32>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum EmbeddingFormat {
        Array(Vec<f32>),
        String(String),
    }

    let value = Option::<EmbeddingFormat>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(EmbeddingFormat::Array(v)) => Ok(Some(v)),
        Some(EmbeddingFormat::String(s)) => {
            let s = s.trim();
            if s.is_empty() || s == "[]" {
                return Ok(None);
            }
            let s = s.trim_start_matches('[').trim_end_matches(']');
            let vec: Result<Vec<f32>, _> = s.split(',').map(|x| x.trim().parse::<f32>()).collect();
            vec.map(Some).map_err(serde::de::Error::custom)
        }
    }
}
