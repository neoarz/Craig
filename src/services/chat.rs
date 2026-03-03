use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::{Context, Data, Error};

const DISCORD_MESSAGE_CHAR_LIMIT: usize = 2000;

#[derive(Debug, Clone, Copy)]
pub enum ChatSource {
    SlashCommand,
    PrefixPrompt,
    MentionMessage,
}

impl ChatSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SlashCommand => "slash",
            Self::PrefixPrompt => "prefix",
            Self::MentionMessage => "mention",
        }
    }
}

pub struct ChatReply {
    pub model_id: String,
    pub content: String,
}

pub fn no_ping_allowed_mentions() -> serenity::CreateAllowedMentions {
    serenity::CreateAllowedMentions::new()
        .all_users(false)
        .all_roles(false)
        .everyone(false)
        .replied_user(false)
}

pub async fn generate_reply(
    data: &Data,
    prompt: &str,
    model: Option<&str>,
) -> Result<ChatReply, Error> {
    let model_id = model.unwrap_or(&data.ai.default_model).to_string();
    let content = data.ai.chat(&model_id, prompt).await?;

    Ok(ChatReply { model_id, content })
}

pub async fn reply_to_context(ctx: Context<'_>, content: &str) -> Result<(), Error> {
    let chunks = split_for_discord(content);

    for (index, chunk) in chunks.into_iter().enumerate() {
        let mut reply = CreateReply::default()
            .content(chunk)
            .allowed_mentions(no_ping_allowed_mentions());
        if index == 0 {
            reply = reply.reply(true);
        }
        ctx.send(reply).await?;
    }

    Ok(())
}

pub async fn reply_to_message(
    http: &serenity::Http,
    msg: &serenity::Message,
    content: &str,
) -> Result<(), Error> {
    let chunks = split_for_discord(content);

    for (index, chunk) in chunks.into_iter().enumerate() {
        let mut builder = serenity::CreateMessage::new()
            .content(chunk)
            .allowed_mentions(no_ping_allowed_mentions());
        if index == 0 {
            builder = builder.reference_message(msg);
        }

        msg.channel_id.send_message(http, builder).await?;
    }

    Ok(())
}

pub fn normalize_prefix_prompt(content: &str) -> Option<&str> {
    let prompt = content.trim();
    (!prompt.is_empty()).then_some(prompt)
}

pub fn extract_mention_prompt(content: &str, bot_id: serenity::UserId) -> Option<String> {
    let id = bot_id.get();
    let sanitized = content
        .replace(&format!("<@{id}>"), " ")
        .replace(&format!("<@!{id}>"), " ");
    let prompt = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");

    (!prompt.is_empty()).then_some(prompt)
}

fn split_for_discord(content: &str) -> Vec<String> {
    if content.is_empty() {
        return vec![" ".to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut len = 0usize;

    for ch in content.chars() {
        if len == DISCORD_MESSAGE_CHAR_LIMIT {
            chunks.push(current);
            current = String::new();
            len = 0;
        }

        current.push(ch);
        len += 1;
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}
