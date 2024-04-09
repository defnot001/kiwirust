use std::net::{Ipv4Addr, SocketAddr};

use rcon::Builder;

use crate::config::{Config, ServerChoice};

pub async fn run_rcon_command(
    server: ServerChoice,
    config: &Config,
    commands: Vec<impl Into<String>>,
) -> anyhow::Result<Vec<Option<String>>> {
    let commands = commands
        .into_iter()
        .map(|c| c.into())
        .collect::<Vec<String>>();

    let server_config = config.minecraft.get(server);

    let host = server_config.host.as_str().parse::<Ipv4Addr>()?;
    let addr = SocketAddr::new(host.into(), server_config.rcon_port);

    let mut connection = Builder::new()
        .enable_minecraft_quirks(true)
        .connect(addr, &server_config.rcon_password)
        .await?;

    let mut responses: Vec<Option<String>> = Vec::new();

    for cmd in commands {
        let response = connection.cmd(cmd.as_str()).await;

        match response {
            Ok(response) => {
                tracing::info!("Command \"{}\" executed successfully on {server}.", &cmd,);

                if response.is_empty() {
                    responses.push(None);
                } else {
                    responses.push(Some(response));
                }
            }
            Err(e) => {
                tracing::error!("Error executing command \"{}\" on {server}: {e}", &cmd,);
                anyhow::bail!("Error executing command \"{}\" on {server}", &cmd);
            }
        }
    }

    Ok(responses)
}
