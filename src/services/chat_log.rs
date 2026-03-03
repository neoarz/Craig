use std::time::Instant;

use poise::serenity_prelude as serenity;

use crate::db::ChatLog;
use crate::services::chat::ChatSource;

pub fn from_message(
    msg: &serenity::Message,
    source: ChatSource,
    model: &str,
    prompt: &str,
    response: &str,
    started: Instant,
    error_text: Option<String>,
) -> ChatLog {
    ChatLog {
        user_id: msg.author.id.get() as i64,
        username: msg.author.name.clone(),
        guild_id: msg.guild_id.map(|id| id.get() as i64),
        channel_id: msg.channel_id.get() as i64,
        source: source.as_str(),
        model: model.to_string(),
        prompt: prompt.to_string(),
        response: response.to_string(),
        latency_ms: elapsed_ms(started),
        success: error_text.is_none(),
        error_text,
    }
}

pub fn from_slash_context(
    ctx: &crate::Context<'_>,
    source: ChatSource,
    model: &str,
    prompt: &str,
    response: &str,
    started: Instant,
    error_text: Option<String>,
) -> ChatLog {
    ChatLog {
        user_id: ctx.author().id.get() as i64,
        username: ctx.author().name.clone(),
        guild_id: ctx.guild_id().map(|id| id.get() as i64),
        channel_id: ctx.channel_id().get() as i64,
        source: source.as_str(),
        model: model.to_string(),
        prompt: prompt.to_string(),
        response: response.to_string(),
        latency_ms: elapsed_ms(started),
        success: error_text.is_none(),
        error_text,
    }
}

fn elapsed_ms(started: Instant) -> i32 {
    started.elapsed().as_millis().min(i32::MAX as u128) as i32
}
