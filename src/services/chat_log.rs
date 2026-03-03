use std::time::Instant;

use poise::serenity_prelude as serenity;

use crate::Context;
use crate::db::{self, ChatLog};

#[derive(Debug, Clone)]
pub struct ChatLogMeta {
    user_id: i64,
    username: String,
    guild_id: Option<i64>,
    channel_id: i64,
    source: String,
}

impl ChatLogMeta {
    pub fn from_context(ctx: &Context<'_>, source: &str) -> Option<Self> {
        let user_id = convert_id("user", ctx.author().id.get())?;
        let guild_id = match ctx.guild_id() {
            Some(id) => Some(convert_id("guild", id.get())?),
            None => None,
        };
        let channel_id = convert_id("channel", ctx.channel_id().get())?;

        Some(Self {
            user_id,
            username: ctx.author().name.clone(),
            guild_id,
            channel_id,
            source: source.to_string(),
        })
    }

    pub fn from_message(msg: &serenity::Message, source: &str) -> Option<Self> {
        let user_id = convert_id("user", msg.author.id.get())?;
        let guild_id = match msg.guild_id {
            Some(id) => Some(convert_id("guild", id.get())?),
            None => None,
        };
        let channel_id = convert_id("channel", msg.channel_id.get())?;

        Some(Self {
            user_id,
            username: msg.author.name.clone(),
            guild_id,
            channel_id,
            source: source.to_string(),
        })
    }

    pub fn success(&self, model: &str, prompt: &str, response: &str, started: Instant) -> ChatLog {
        ChatLog {
            user_id: self.user_id,
            username: self.username.clone(),
            guild_id: self.guild_id,
            channel_id: self.channel_id,
            source: self.source.clone(),
            model: model.to_string(),
            prompt: prompt.to_string(),
            response: response.to_string(),
            latency_ms: elapsed_ms_i32(started),
            success: true,
            error_text: None,
        }
    }

    pub fn failure(
        &self,
        model: &str,
        prompt: &str,
        response: &str,
        error_text: impl Into<String>,
        started: Instant,
    ) -> ChatLog {
        ChatLog {
            user_id: self.user_id,
            username: self.username.clone(),
            guild_id: self.guild_id,
            channel_id: self.channel_id,
            source: self.source.clone(),
            model: model.to_string(),
            prompt: prompt.to_string(),
            response: response.to_string(),
            latency_ms: elapsed_ms_i32(started),
            success: false,
            error_text: Some(error_text.into()),
        }
    }
}

fn elapsed_ms_i32(started: Instant) -> i32 {
    started.elapsed().as_millis().min(i32::MAX as u128) as i32
}

fn convert_id(field: &'static str, id: u64) -> Option<i64> {
    match db::try_snowflake_to_i64(id) {
        Ok(value) => Some(value),
        Err(_) => {
            crate::app_error!("Cannot write chat log: {field} id {id} is outside i64 range");
            None
        }
    }
}
