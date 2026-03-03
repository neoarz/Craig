use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;
use sqlx::PgPool;

use crate::commands::{ai::ai, ping::ping, sync as sync_commands};
use crate::config::AppConfig;
use crate::helpers;
use crate::services::{chat, dispatch};
use crate::{Data, Error};

const INTENTS: GatewayIntents =
    GatewayIntents::non_privileged().union(serenity::GatewayIntents::MESSAGE_CONTENT);

pub async fn start(config: AppConfig, data: Data) -> Result<(), Error> {
    let AppConfig {
        token,
        prefix,
        dev_guild_id,
        ..
    } = config;
    let db = data.db.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            on_error: dispatch::on_error,
            event_handler: dispatch::on_event,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(prefix),
                mention_as_prefix: false,
                ..Default::default()
            },
            allowed_mentions: Some(chat::no_ping_allowed_mentions()),
            command_check: Some(|ctx| Box::pin(helpers::ensure_guild_allowed(ctx))),
            commands: vec![ai(), ping(), sync_commands::sync(), sync_commands::unsync()],
            post_command: |ctx| Box::pin(log_command(ctx)),
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(setup(ctx, ready, framework, dev_guild_id, data))
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, INTENTS)
        .framework(framework)
        .await?;

    spawn_shutdown_handler(client.shard_manager.clone(), db);

    client.start().await?;
    Ok(())
}

async fn setup(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: &poise::Framework<Data, Error>,
    dev_guild_id: serenity::GuildId,
    data: Data,
) -> Result<Data, Error> {
    poise::builtins::register_in_guild(ctx, &framework.options().commands, dev_guild_id).await?;

    crate::bot_info!("Logged in as {} (ID: {})", ready.user.tag(), ready.user.id);

    match dev_guild_id.to_partial_guild(&ctx.http).await {
        Ok(guild) => crate::bot_info!("Dev Guild: {} (ID: {})", guild.name, guild.id),
        Err(error) => crate::bot_warn!(
            "Could not use dev guild {} for startup logging: {}",
            dev_guild_id,
            error
        ),
    }

    match ctx.http.get_current_application_info().await {
        Ok(app) => {
            if let Some(team) = app.team {
                crate::bot_info!("Team: {}", team.name);
            }
        }
        Err(error) => crate::bot_warn!("Could not fetch application/team info: {}", error),
    }

    crate::bot_info!("Commands: {}", framework.options().commands.len());
    Ok(data)
}

async fn log_command(ctx: poise::Context<'_, Data, Error>) {
    let command = format!("{}{}", ctx.prefix(), ctx.invoked_command_name());
    let location = dispatch::format_location(ctx.guild_id(), ctx.channel_id());

    crate::run_debug!(
        "Executed {} in {} by {} (ID: {})",
        command,
        location,
        ctx.author().name,
        ctx.author().id.get()
    );
}

fn spawn_shutdown_handler(shard_manager: std::sync::Arc<serenity::ShardManager>, db: PgPool) {
    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            crate::app_error!("Cannot register ctrl+c handler: {error}");
            return;
        }

        crate::bot_info!("Shutting down the bot");
        shard_manager.shutdown_all().await;
        db.close().await;
    });
}
