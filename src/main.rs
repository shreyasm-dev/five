use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;

struct Bot;

#[async_trait]
impl EventHandler for Bot {
  async fn ready(&self, _: Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);
  }
}

#[shuttle_runtime::main]
async fn serenity(
  #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
  let token = secrets
    .get("DISCORD_TOKEN")
    .context("DISCORD_TOKEN was not found")?;

  let intents = GatewayIntents::empty();

  let client = Client::builder(&token, intents)
    .event_handler(Bot)
    .await
    .expect("error creating client");

  Ok(client.into())
}
