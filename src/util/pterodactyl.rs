use std::fmt::Display;

use anyhow::Context;
use pterodactyl_api::client::{
    backups::{Backup, BackupParams},
    ClientBuilder, ServerState,
};
use serenity::all::User;
use uuid::Uuid;

use crate::config::{PterodactylConfig, ServerChoice, ServerConfig};

pub struct PteroClient;
pub struct DisplayServerState(pub ServerState);

impl PteroClient {
    pub async fn server_state(
        ptero_config: &PterodactylConfig,
        server_config: &ServerConfig,
    ) -> anyhow::Result<ServerState> {
        let state = ClientBuilder::new(ptero_config.url.clone(), ptero_config.api_key.clone())
            .build()
            .get_server(server_config.panel_id.clone())
            .get_resources()
            .await?
            .current_state;

        Ok(state)
    }

    pub async fn backup_list(
        ptero_config: &PterodactylConfig,
        server_config: &ServerConfig,
    ) -> anyhow::Result<Vec<Backup>> {
        let backups = ClientBuilder::new(ptero_config.url.clone(), ptero_config.api_key.clone())
            .build()
            .get_server(server_config.panel_id.clone())
            .list_backups()
            .await?;

        Ok(backups)
    }

    pub async fn backup_details(
        ptero_config: &PterodactylConfig,
        server_config: &ServerConfig,
        uuid: Uuid,
    ) -> anyhow::Result<Backup> {
        let backup = ClientBuilder::new(ptero_config.url.clone(), ptero_config.api_key.clone())
            .build()
            .get_server(server_config.panel_id.clone())
            .get_backup(uuid)
            .await?;

        Ok(backup)
    }

    pub async fn create_backup_and_wait(
        ptero_config: &PterodactylConfig,
        server_config: &ServerConfig,
        backup_name: Option<String>,
        locked: Option<bool>,
        user: &User,
    ) -> anyhow::Result<Backup> {
        let locked = locked.unwrap_or(false);
        let name = backup_name.unwrap_or(format!("Discord Bot: ({})", user.name));

        let options = if locked {
            BackupParams::new().with_name(name).set_locked()
        } else {
            BackupParams::new().with_name(name)
        };

        let client =
            ClientBuilder::new(ptero_config.url.clone(), ptero_config.api_key.clone()).build();

        let server = client.get_server(server_config.panel_id.clone());
        let created_backup = server.create_backup_with_params(options).await?;

        let mut count = 0;

        while count < 45 {
            let polled = server.get_backup(created_backup.uuid).await?;

            if polled.completed_at.is_some() {
                return Ok(polled);
            }

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            count += 1;
        }

        Ok(created_backup)
    }

    pub async fn delete_backup(
        ptero_config: &PterodactylConfig,
        server_config: &ServerConfig,
        uuid: Uuid,
    ) -> anyhow::Result<()> {
        let server_choice = ServerChoice::try_from(server_config)?;

        ClientBuilder::new(ptero_config.url.clone(), ptero_config.api_key.clone())
            .build()
            .get_server(server_config.panel_id.clone())
            .delete_backup(uuid)
            .await
            .context(format!(
                "Failed to delete backup with id {uuid} from {server_choice}"
            ))
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
