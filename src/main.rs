use anyhow::Context as _;
use poise::{
  serenity_prelude::{
    ChannelType, ClientBuilder, CreateEmbed, CreateThread, Error, GatewayIntents, GuildId,
    Mentionable,
  },
  CreateReply, Framework, FrameworkOptions,
};
use rand::Rng;
use shuttle_runtime::SecretStore;
use std::vec;

type Context<'a> = poise::Context<'a, (), Error>;

#[poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
  ctx.say("Pong!").await?;
  Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn roll(
  ctx: Context<'_>,
  #[description = "Number of sides"] sides: Option<u32>,
) -> Result<(), Error> {
  let sides = sides.unwrap_or(6);
  let roll = rand::rng().random_range(1..=sides);

  let name = ctx.guild_id().map(|guild_id| async move {
    ctx
      .author()
      .nick_in(ctx.http(), guild_id)
      .await
      .unwrap_or(ctx.author().display_name().to_string())
  });

  let name = if let Some(name) = name {
    name.await
  } else {
    ctx.author().display_name().to_string()
  };

  ctx
    .send(CreateReply::default().embed(
      CreateEmbed::new().description(format!("# {}\n-# {} sides (**{}**)", roll, sides, name)),
    ))
    .await?;

  Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn thread(
  ctx: Context<'_>,
  #[description = "Thread name"] name: String,
  #[description = "Private thread"] private: Option<bool>,
) -> Result<(), Error> {
  let channel = ctx
    .channel_id()
    .create_thread(
      &ctx.http(),
      CreateThread::new(name).kind(if private.unwrap_or(false) {
        ChannelType::PrivateThread
      } else {
        ChannelType::PublicThread
      }),
    )
    .await?;

  ctx
    .send(
      CreateReply::default().embed(CreateEmbed::new().description(format!(
        "{} created: {}",
        if private.unwrap_or(false) {
          "Private thread"
        } else {
          "Thread"
        },
        channel.mention()
      ))),
    )
    .await?;

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
      commands: vec![ping(), roll(), thread()],
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
