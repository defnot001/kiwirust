use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O Error: {0}")]
    Io(#[from] io::Error),
    #[error("HTTP Request Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Join Error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Discord Error: {0}")]
    Serenity(#[from] serenity::Error),
    #[error("Utf8 Error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Pterodactyl Error: {0}")]
    Pterodactyl(#[from] pterodactyl_api::Error),
    // #[error("Other Error: {0}")]
    // Other(String),
}
