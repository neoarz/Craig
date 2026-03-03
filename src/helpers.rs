use poise::CreateReply;

use crate::{Context, Error};

pub const UNAUTHORIZED_REPLY: &str =
    "This bot is not authorized for this server. Contact neoarz on Discord to request access.";

pub(crate) async fn send_ephemeral(
    ctx: Context<'_>,
    content: impl Into<String>,
) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .content(content.into())
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

pub(crate) async fn ensure_owner(ctx: Context<'_>) -> Result<bool, Error> {
    let is_owner = ctx.framework().options().owners.contains(&ctx.author().id);
    if is_owner {
        return Ok(true);
    }

    send_ephemeral(ctx, "Only the bot owner can use this command.").await?;
    Ok(false)
}

pub(crate) async fn ensure_guild_allowed(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return Ok(true),
    };

    if ctx.data().allowed_guilds.contains(&guild_id) {
        return Ok(true);
    }

    send_ephemeral(ctx, UNAUTHORIZED_REPLY).await?;
    Ok(false)
}
