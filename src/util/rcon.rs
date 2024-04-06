use std::net::{Ipv4Addr, SocketAddr};

use rcon::Builder;

use crate::config::{Config, ServerChoice};

pub async fn run_rcon_command(
    server: &ServerChoice,
    config: &Config,
    command: impl Into<String>,
) -> anyhow::Result<Option<String>> {
    let command = command.into();

    let server_config = match server {
        ServerChoice::Smp => &config.minecraft.smp,
        ServerChoice::Cmp => &config.minecraft.cmp,
        ServerChoice::Cmp2 => &config.minecraft.cmp2,
        ServerChoice::Copy => &config.minecraft.copy,
        ServerChoice::Snapshots => &config.minecraft.snapshots,
    };

    let host = config.minecraft.host.as_str().parse::<Ipv4Addr>()?;
    let addr = SocketAddr::new(host.into(), server_config.rcon_port);

    let mut connection = Builder::new()
        .enable_minecraft_quirks(true)
        .connect(addr, &server_config.rcon_password)
        .await?;

    let response = connection.cmd(command.as_str()).await;

    match response {
        Ok(response) => {
            tracing::info!(
                "Command \"{}\" executed successfully on {server}.",
                &command,
            );

            if response.is_empty() {
                Ok(None)
            } else {
                Ok(Some(response))
            }
        }
        Err(e) => {
            tracing::error!("Error executing command \"{}\" on {server}: {e}", &command,);
            Err(anyhow::anyhow!(
                "Error executing command \"{}\" on {server}",
                &command,
            ))
        }
    }
}
