use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("http: {0}")]    Http(#[from] reqwest::Error),
    #[error("json: {0}")]    Json(#[from] serde_json::Error),
    #[error("tg [{code}]: {description}")] Api { code: i64, description: String },
    #[error("{0}")]          Other(String),
}
