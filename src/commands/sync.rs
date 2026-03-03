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
            let guild_id = dev_guild_id(ctx);
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
            let guild_id = dev_guild_id(ctx);
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

fn dev_guild_id(ctx: Context<'_>) -> serenity::GuildId {
    ctx.data().dev_guild_id
}
