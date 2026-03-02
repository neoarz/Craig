use std::sync::Arc;

use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::commands::{help::help, ping::pong};
use crate::config::AppConfig;
use crate::{Data, Error};

const INTENTS: GatewayIntents =
    GatewayIntents::non_privileged().union(serenity::GatewayIntents::MESSAGE_CONTENT);

pub async fn start(config: AppConfig, data: Data, db: PgPool) -> Result<(), Error> {
    let dev_guild_id = config.dev_guild_id;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: config.primary_prefix,
                additional_prefixes: config.additional_prefixes,
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(120),
                ))),
                ..Default::default()
            },
            commands: vec![help(), pong()],
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                info!("Logged in as {} (ID: {})", ready.user.tag(), ready.user.id);

                if let Some(guild_id) = dev_guild_id {
                    match guild_id.to_partial_guild(&ctx.http).await {
                        Ok(guild) => info!("Dev Guild: {} (ID: {})", guild.name, guild.id),
                        Err(error) => warn!(
                            "Could not resolve dev guild {} for startup logging: {}",
                            guild_id, error
                        ),
                    }
                }
                // Most people dont use team but printing in case someone does (this shouldnt show if the bot is not in a team)
                match ctx.http.get_current_application_info().await {
                    Ok(application) => {
                        if let Some(team) = application.team {
                            info!("Team: {}", team.name);
                        }
                    }
                    Err(error) => warn!("Could not fetch application/team info: {}", error),
                }

                info!("Commands: {}", framework.options().commands.len());
                Ok(data)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(config.token, INTENTS)
        .framework(framework)
        .await?;

    spawn_shutdown_handler(client.shard_manager.clone(), db);

    client.start().await?;
    Ok(())
}

fn spawn_shutdown_handler(shard_manager: Arc<serenity::ShardManager>, db: PgPool) {
    tokio::spawn(async move {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!("Cannot register ctrl+c handler: {error}");
            return;
        }

        tracing::info!("Shutting down the bot");
        shard_manager.shutdown_all().await;
        db.close().await;
    });
}
