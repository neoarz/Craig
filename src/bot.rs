use std::sync::Arc;

use poise::serenity_prelude as serenity;
use serenity::GatewayIntents;
use sqlx::PgPool;

use crate::commands::{help::help, ping::pong};
use crate::config::AppConfig;
use crate::{Data, Error};

const INTENTS: GatewayIntents =
    GatewayIntents::non_privileged().union(serenity::GatewayIntents::MESSAGE_CONTENT);

pub async fn start(config: AppConfig, data: Data, db: PgPool) -> Result<(), Error> {
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
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(config.token, INTENTS)
        .framework(framework)
        .await?;

    spawn_shutdown_handler(client.shard_manager.clone(), db);

    tracing::info!("Starting the bot");
    client.start().await?;
    Ok(())
}

fn spawn_shutdown_handler(shard_manager: Arc<serenity::ShardManager>, db: PgPool) {
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Cannot register a ctrl+c handler!");

        tracing::info!("Shutting down the bot");
        shard_manager.shutdown_all().await;
        db.close().await;
    });
}
