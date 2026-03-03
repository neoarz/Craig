use crate::services::chat::{self, ChatSource};
use crate::{Context, Error};

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum AiModel {
    #[name = "Llama 3.3 70B"]
    Llama3_3_70b,
    #[name = "Llama 3 8B"]
    Llama3_8b,
    #[name = "DeepSeek R1 70B"]
    DeepSeekR1_70b,
    #[name = "Mistral Nemo"]
    MistralNemo,
    #[name = "Qwen 3 32B"]
    Qwen3_32b,
    #[name = "GPT OSS 120B"]
    GptOss120b,
    #[name = "GPT OSS 20B"]
    GptOss20b,
}

impl AiModel {
    fn model_id(self) -> &'static str {
        match self {
            Self::Llama3_3_70b => "llama3.3-70b-instruct",
            Self::Llama3_8b => "llama3-8b-instruct",
            Self::DeepSeekR1_70b => "deepseek-r1-distill-llama-70b",
            Self::MistralNemo => "mistral-nemo-instruct-2407",
            Self::Qwen3_32b => "alibaba-qwen3-32b",
            Self::GptOss120b => "openai-gpt-oss-120b",
            Self::GptOss20b => "openai-gpt-oss-20b",
        }
    }
}

/// Ask the AI a question
#[poise::command(slash_command)]
pub async fn ai(
    ctx: Context<'_>,
    #[description = "Which AI model to use"] model: Option<AiModel>,
    #[description = "Your question or prompt"]
    #[rest]
    prompt: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let selected_model = model.map(AiModel::model_id);
    let reply = chat::generate_reply(ctx.data(), &prompt, selected_model).await?;
    chat::reply_to_context(ctx, &reply.content).await?;

    crate::run_debug!(
        "Executed AI chat [{}] in slash command by {} (ID: {}) using model {}",
        ChatSource::SlashCommand.as_str(),
        ctx.author().name,
        ctx.author().id.get(),
        reply.model_id
    );

    Ok(())
}
