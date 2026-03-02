use std::env;

use poise::serenity_prelude as serenity;

use crate::helpers::{ensure_owner, send_ephemeral};
use crate::{Context, Error};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
enum SyncScope {
    #[name = "guild"]
    Guild,
    #[name = "global"]
    Global,
}

async fn require_dev_guild_id(ctx: Context<'_>) -> Result<Option<serenity::GuildId>, Error> {
    match env::var("DEV_GUILD_ID") {
        Ok(raw_id) => match raw_id.parse::<u64>() {
            Ok(parsed_id) => Ok(Some(serenity::GuildId::new(parsed_id))),
            Err(_) => {
                send_ephemeral(ctx, "DEV_GUILD_ID is invalid.").await?;
                Ok(None)
            }
        },
        Err(_) => {
            send_ephemeral(ctx, "DEV_GUILD_ID is missing.").await?;
            Ok(None)
        }
    }
}

/// Sync slash commands to guild or globally.
#[poise::command(slash_command)]
pub async fn sync(
    ctx: Context<'_>,
    #[description = "Where to apply command changes"] scope: SyncScope,
) -> Result<(), Error> {
    if !ensure_owner(ctx).await? {
        return Ok(());
    }

    let commands = &ctx.framework().options().commands;
    let synced_count = poise::builtins::create_application_commands(commands).len();

    match scope {
        SyncScope::Guild => {
            let guild_id = match require_dev_guild_id(ctx).await? {
                Some(guild_id) => guild_id,
                None => return Ok(()),
            };
            poise::builtins::register_in_guild(ctx, commands, guild_id).await?;
            send_ephemeral(
                ctx,
                format!(
                    "Synced {synced_count} commands to guild {}.",
                    guild_id.get()
                ),
            )
            .await?;
        }
        SyncScope::Global => {
            poise::builtins::register_globally(ctx, commands).await?;
            send_ephemeral(ctx, format!("Synced {synced_count} commands globally.")).await?;
        }
    }

    Ok(())
}

/// Unsync slash commands from guild or globally.
#[poise::command(slash_command)]
pub async fn unsync(
    ctx: Context<'_>,
    #[description = "Where to apply command changes"] scope: SyncScope,
) -> Result<(), Error> {
    if !ensure_owner(ctx).await? {
        return Ok(());
    }

    match scope {
        SyncScope::Guild => {
            let guild_id = match require_dev_guild_id(ctx).await? {
                Some(guild_id) => guild_id,
                None => return Ok(()),
            };
            guild_id
                .set_commands(ctx, Vec::<serenity::CreateCommand>::new())
                .await?;
            send_ephemeral(
                ctx,
                format!(
                    "Removed all application commands from guild {}.",
                    guild_id.get()
                ),
            )
            .await?;
        }
        SyncScope::Global => {
            serenity::Command::set_global_commands(ctx, Vec::<serenity::CreateCommand>::new())
                .await?;
            send_ephemeral(ctx, "Removed all global application commands.").await?;
        }
    }

    Ok(())
}
