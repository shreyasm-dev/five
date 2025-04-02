use anyhow::Context as _;
use poise::{
  serenity_prelude::{ClientBuilder, Error, GatewayIntents, GuildId, User},
  Framework, FrameworkOptions,
};
use shuttle_runtime::SecretStore;

type Context<'a> = poise::Context<'a, (), Error>;

#[poise::command(slash_command, prefix_command)]
async fn age(
  ctx: Context<'_>,
  #[description = "Selected user"] user: Option<User>,
) -> Result<(), Error> {
  let u = user.as_ref().unwrap_or_else(|| ctx.author());
  let response = format!("{}'s account was created at {}", u.name, u.created_at());
  ctx.say(response).await?;
  Ok(())
}

#[shuttle_runtime::main]
async fn serenity(
  #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
  let token = secrets
    .get("DISCORD_TOKEN")
    .context("DISCORD_TOKEN was not found")?;
  let guild_id = GuildId::from(
    secrets
      .get("GUILD_ID")
      .context("GUILD_ID was not found")?
      .parse::<u64>()
      .context("GUILD_ID is not a valid u64")?,
  );

  let intents = GatewayIntents::empty();

  let framework = Framework::<(), Error>::builder()
    .options(FrameworkOptions {
      commands: vec![age()],
      ..Default::default()
    })
    .setup(move |ctx, _, framework: &Framework<_, _>| {
      Box::pin(async move {
        poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
        Ok(())
      })
    })
    .build();

  let client = ClientBuilder::new(token, intents)
    .framework(framework)
    .await
    .context("failed to create client")?;

  Ok(client.into())
}
