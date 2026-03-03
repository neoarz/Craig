use std::sync::Arc;
use std::time::Instant;

use poise::serenity_prelude as serenity;

use crate::db;
use crate::services::chat::{self, ChatSource};
use crate::services::chat_log::ChatLogMeta;
use crate::{Data, Error};

pub fn on_error(error: poise::FrameworkError<'_, Data, Error>) -> poise::BoxFuture<'_, ()> {
    Box::pin(async move {
        match error {
            poise::FrameworkError::UnknownCommand {
                ctx,
                msg,
                prefix,
                msg_content,
                framework,
                ..
            } => {
                let configured_prefix = framework.options.prefix_options.prefix.as_deref();
                if configured_prefix != Some(prefix) {
                    crate::bot_warn!(
                        "Recognized prefix `{}`, but didn't recognize command name in `{}`",
                        prefix,
                        msg_content
                    );
                    return;
                }

                let prompt = match chat::normalize_prefix_prompt(msg_content) {
                    Some(prompt) => prompt,
                    None => {
                        let guidance = format!("Type a message after `{prefix}` to chat with me.");
                        if let Err(error) = chat::reply_to_message(&ctx.http, msg, &guidance).await
                        {
                            crate::app_error!("Failed to send prefix guidance reply: {error}");
                        }
                        return;
                    }
                };

                handle_ai_chat(
                    &ctx.http,
                    msg,
                    framework.user_data,
                    prompt,
                    ChatSource::PrefixPrompt,
                )
                .await;
            }
            other => {
                if let Err(error) = poise::builtins::on_error(other).await {
                    crate::app_error!("Error while handling framework error: {error}");
                }
            }
        }
    })
}

pub fn on_event<'a>(
    ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    framework: poise::FrameworkContext<'a, Data, Error>,
    _data: &'a Data,
) -> poise::BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        let serenity::FullEvent::Message { new_message } = event else {
            return Ok(());
        };

        if new_message.author.bot || new_message.author.id == framework.bot_id {
            return Ok(());
        }

        let is_prefix_message = framework
            .options
            .prefix_options
            .prefix
            .as_deref()
            .is_some_and(|p| new_message.content.trim_start().starts_with(p));
        if is_prefix_message {
            return Ok(());
        }

        let is_mention = new_message
            .mentions
            .iter()
            .any(|user| user.id == framework.bot_id);
        if !is_mention {
            return Ok(());
        }

        let prompt = match chat::extract_mention_prompt(&new_message.content, framework.bot_id) {
            Some(prompt) => prompt,
            None => return Ok(()),
        };

        handle_ai_chat(
            &ctx.http,
            new_message,
            framework.user_data,
            &prompt,
            ChatSource::MentionMessage,
        )
        .await;

        Ok(())
    })
}

pub(crate) fn format_location(
    guild_id: Option<serenity::GuildId>,
    channel_id: serenity::ChannelId,
) -> String {
    match guild_id {
        Some(id) => format!("(Guild: {}) | (Channel: {})", id.get(), channel_id.get()),
        None => "DMs".to_string(),
    }
}

async fn handle_ai_chat(
    http: &Arc<serenity::Http>,
    msg: &serenity::Message,
    data: &Data,
    prompt: &str,
    source: ChatSource,
) {
    let started = Instant::now();
    let _typing = msg.channel_id.start_typing(http);
    let source_name = source.as_str();
    let log_meta = ChatLogMeta::from_message(msg, source_name);
    let user_id = msg.author.id.get();
    let username = msg.author.name.clone();

    match chat::generate_reply(data, prompt, None).await {
        Ok(reply) => {
            match chat::reply_to_message(http.as_ref(), msg, &reply.content).await {
                Ok(()) => {
                    if let Some(meta) = log_meta.as_ref() {
                        let chat_log =
                            meta.success(&reply.model_id, prompt, &reply.content, started);
                        db::insert_chat_log(&data.db, &chat_log).await;
                    }
                }
                Err(error) => {
                    crate::app_error!("Failed to send {} AI reply: {error}", source_name);
                    if let Some(meta) = log_meta.as_ref() {
                        let chat_log = meta.failure(
                            &reply.model_id,
                            prompt,
                            &reply.content,
                            error.to_string(),
                            started,
                        );
                        db::insert_chat_log(&data.db, &chat_log).await;
                    }
                    return;
                }
            }

            let location = format_location(msg.guild_id, msg.channel_id);
            crate::run_debug!(
                "Executed AI chat [{}] in {} by {} (ID: {}) using (Model: {}) in {}ms",
                source_name,
                location,
                username,
                user_id,
                reply.model_id,
                started.elapsed().as_millis()
            );
        }
        Err(error) => {
            let mut error_text = error.to_string();
            crate::app_error!("AI {} chat failed: {error_text}", source_name);
            if let Err(send_error) = chat::reply_to_message(
                http.as_ref(),
                msg,
                "I couldn't process that right now. Please try again in a moment.",
            )
            .await
            {
                crate::app_error!(
                    "Failed to send fallback error for {} chat: {send_error}",
                    source_name
                );
                error_text = format!("{error_text}; fallback_send_failed: {send_error}");
            }

            if let Some(meta) = log_meta.as_ref() {
                let chat_log =
                    meta.failure(&data.ai.default_model, prompt, "", error_text, started);
                db::insert_chat_log(&data.db, &chat_log).await;
            }
        }
    }
}
