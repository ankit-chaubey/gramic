use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Update {
    pub update_id:      i64,
    #[serde(default)] pub message:        Option<Message>,
    #[serde(default)] pub edited_message: Option<Message>,
    #[serde(default)] pub channel_post:   Option<Message>,
    #[serde(default)] pub callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub chat:       Chat,
    #[serde(default)] pub from: Option<User>,
    #[serde(default)] pub text: Option<String>,
    #[serde(default)] pub date: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Chat {
    pub id:   i64,
    #[serde(rename = "type")] pub kind: String,
    #[serde(default)] pub username:   Option<String>,
    #[serde(default)] pub title:      Option<String>,
    #[serde(default)] pub first_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id:         i64,
    pub first_name: String,
    #[serde(default)] pub username: Option<String>,
    #[serde(default)] pub is_bot:   bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    #[serde(default)] pub from:    Option<User>,
    #[serde(default)] pub data:    Option<String>,
    #[serde(default)] pub message: Option<Message>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookInfo {
    pub url:                    String,
    pub has_custom_certificate: bool,
    pub pending_update_count:   i64,
    #[serde(default)] pub ip_address:                      Option<String>,
    #[serde(default)] pub last_error_date:                 Option<i64>,
    #[serde(default)] pub last_error_message:              Option<String>,
    #[serde(default)] pub last_synchronization_error_date: Option<i64>,
    #[serde(default)] pub max_connections:                 Option<i64>,
    #[serde(default)] pub allowed_updates:                 Option<Vec<String>>,
}
