use std::sync::Arc;

use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;
use sqlx::PgPool;

use crate::commands::{ping::ping, sync as sync_commands};
use crate::config::AppConfig;
use crate::{Data, Error};

const INTENTS: GatewayIntents =
    GatewayIntents::non_privileged().union(serenity::GatewayIntents::MESSAGE_CONTENT);

pub async fn start(config: AppConfig, data: Data, db: PgPool) -> Result<(), Error> {
    let AppConfig {
        token,
        prefix,
        dev_guild_id,
        ..
    } = config;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(prefix),
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(120),
                ))),
                ..Default::default()
            },
            commands: vec![ping(), sync_commands::sync(), sync_commands::unsync()],
            post_command: |ctx| {
                Box::pin(async move {
                    let command_name = format!("{}{}", ctx.prefix(), ctx.invoked_command_name());
                    let location = match ctx.partial_guild().await {
                        Some(guild) => format!("{} (ID: {})", guild.name, guild.id.get()),
                        None => "DMs".to_string(),
                    };

                    crate::run_debug!(
                        "Executed {} in {} by {} (ID: {})",
                        command_name,
                        location,
                        ctx.author().name,
                        ctx.author().id.get()
                    );
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                let dev_guild_id =
                    dev_guild_id.ok_or("DEV_GUILD_ID isnt found, where it at buddy")?;
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    dev_guild_id,
                )
                .await?;

                crate::bot_info!("Logged in as {} (ID: {})", ready.user.tag(), ready.user.id);

                match dev_guild_id.to_partial_guild(&ctx.http).await {
                    Ok(guild) => crate::bot_info!("Dev Guild: {} (ID: {})", guild.name, guild.id),
                    Err(error) => crate::bot_warn!(
                        "Could not use dev guild {} for startup logging: {}",
                        dev_guild_id,
                        error
                    ),
                }
                // Most people dont use team but printing in case someone does (this shouldnt show if the bot is not in a team)
                match ctx.http.get_current_application_info().await {
                    Ok(application) => {
                        if let Some(team) = application.team {
                            crate::bot_info!("Team: {}", team.name);
                        }
                    }
                    Err(error) => {
                        crate::bot_warn!("Could not fetch application/team info: {}", error)
                    }
                }

                crate::bot_info!("Commands: {}", framework.options().commands.len());
                Ok(data)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, INTENTS)
        .framework(framework)
        .await?;

    spawn_shutdown_handler(client.shard_manager.clone(), db);

    client.start().await?;
    Ok(())
}

fn spawn_shutdown_handler(shard_manager: Arc<serenity::ShardManager>, db: PgPool) {
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
