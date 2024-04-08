use std::fmt::Display;

use pterodactyl_api::client::{ClientBuilder, ServerState};

use crate::config::{PterodactylConfig, ServerConfig};

pub struct PteroClient;
pub struct DisplayServerState(pub ServerState);

impl PteroClient {
    pub async fn server_state(
        ptero_config: &PterodactylConfig,
        server_config: &ServerConfig,
    ) -> anyhow::Result<ServerState> {
        Ok(
            ClientBuilder::new(ptero_config.url.clone(), ptero_config.api_key.clone())
                .build()
                .get_server(server_config.panel_id.clone())
                .get_resources()
                .await?
                .current_state,
        )
    }
}

impl Display for DisplayServerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ServerState::Offline => write!(f, "offline"),
            ServerState::Running => write!(f, "running"),
            ServerState::Starting => write!(f, "starting"),
            ServerState::Stopping => write!(f, "stopping"),
        }
    }
}
