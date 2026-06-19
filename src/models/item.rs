use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub name: String,
    pub description: Option<String>,
}

impl CreateItemRequest {
    pub fn validate(&self) -> Result<(), String> {
        let trimmed_name = self.name.trim();
        if trimmed_name.is_empty() {
            return Err("name is required".to_owned());
        }

        if trimmed_name.len() > 120 {
            return Err("name must be 120 characters or less".to_owned());
        }

        if self
            .description
            .as_ref()
            .is_some_and(|description| description.len() > 1000)
        {
            return Err("description must be 1000 characters or less".to_owned());
        }

        Ok(())
    }

    pub fn sanitized_name(&self) -> String {
        self.name.trim().to_owned()
    }

    pub fn sanitized_description(&self) -> Option<String> {
        self.description
            .as_ref()
            .map(|description| description.trim().to_owned())
            .filter(|description| !description.is_empty())
    }
}