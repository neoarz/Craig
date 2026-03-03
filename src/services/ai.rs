use std::time::Duration;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
};

use crate::Error;
use crate::config::AiConfig;

const DO_INFERENCE_BASE_URL: &str = "https://inference.do-ai.run/v1";
const SYSTEM_PROMPT: &str = include_str!("../../CRAIG.md");
const AI_CONNECT_TIMEOUT_SECS: u64 = 10;
const AI_REQUEST_TIMEOUT_SECS: u64 = 45;

pub struct AiService {
    client: Client<OpenAIConfig>,
    pub default_model: String,
}

impl AiService {
    pub fn new(config: &AiConfig) -> Result<Self, Error> {
        let openai_config = OpenAIConfig::new()
            .with_api_base(DO_INFERENCE_BASE_URL)
            .with_api_key(&config.model_access_key);
        let http_client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(AI_CONNECT_TIMEOUT_SECS))
            .timeout(Duration::from_secs(AI_REQUEST_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            client: Client::with_config(openai_config).with_http_client(http_client),
            default_model: config.default_model.clone(),
        })
    }

    pub async fn chat(&self, model: &str, user_message: &str) -> Result<String, Error> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(SYSTEM_PROMPT)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_message)
                .build()?
                .into(),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(messages)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let reply = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or("The AI model returned an empty response.")?;

        Ok(reply.clone())
    }
}
