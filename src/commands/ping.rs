use poise::CreateReply;

use crate::{Context, Error};

/// Ping!
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let gateway_ping = ctx.ping().await;

    let rest_started = std::time::Instant::now();
    ctx.http().get_current_user().await?;
    let rest_ping = rest_started.elapsed();

    ctx.send(
        CreateReply::default()
            .content(format!(
                "pong!\nGateway ping: {}ms\nREST ping: {}ms",
                gateway_ping.as_millis(),
                rest_ping.as_millis()
            ))
            .reply(true),
    )
    .await?;

    Ok(())
}
