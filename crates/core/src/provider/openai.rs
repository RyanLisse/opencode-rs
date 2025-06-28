use super::*;
use crate::config::OpenAIConfig;
use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, ChatCompletionRequestAssistantMessageArgs,
        CreateChatCompletionRequestArgs, CreateChatCompletionStreamResponse,
    },
    Client,
};
use futures::StreamExt;

/// OpenAI provider implementation
pub struct OpenAIProvider {
    client: Client<async_openai::config::OpenAIConfig>,
    config: OpenAIConfig,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(api_key: String, config: OpenAIConfig) -> Self {
        let openai_config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(config.api_base.clone());

        Self {
            client: Client::with_config(openai_config),
            config,
        }
    }

    fn convert_messages(&self, messages: Vec<Message>) -> Vec<ChatCompletionRequestMessage> {
        messages
            .into_iter()
            .map(|msg| match msg.role.as_str() {
                "system" => ChatCompletionRequestSystemMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .unwrap()
                    .into(),
                "assistant" => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .unwrap()
                    .into(),
                _ => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content)
                    .build()
                    .unwrap()
                    .into(),
            })
            .collect()
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let mut builder = CreateChatCompletionRequestArgs::default();
        builder
            .model(&request.model)
            .messages(self.convert_messages(request.messages));

        if let Some(temp) = request.temperature {
            builder.temperature(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            builder.max_tokens(max_tokens as u16);
        }

        let openai_request = builder
            .build()
            .map_err(|e| Error::Provider(format!("Failed to build request: {}", e)))?;

        let response = self
            .client
            .chat()
            .create(openai_request)
            .await
            .map_err(|e| Error::Provider(format!("OpenAI API error: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| Error::Provider("No content in response".into()))?
            .clone();

        Ok(CompletionResponse {
            content,
            model: response.model,
            usage: Usage {
                prompt_tokens: response.usage.as_ref().map(|u| u.prompt_tokens).unwrap_or(0) as u32,
                completion_tokens: response
                    .usage
                    .as_ref()
                    .map(|u| u.completion_tokens)
                    .unwrap_or(0) as u32,
                total_tokens: response.usage.as_ref().map(|u| u.total_tokens).unwrap_or(0) as u32,
            },
        })
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        let mut builder = CreateChatCompletionRequestArgs::default();
        builder
            .model(&request.model)
            .messages(self.convert_messages(request.messages))
            .stream(true);

        if let Some(temp) = request.temperature {
            builder.temperature(temp);
        }

        if let Some(max_tokens) = request.max_tokens {
            builder.max_tokens(max_tokens as u16);
        }

        let openai_request = builder
            .build()
            .map_err(|e| Error::Provider(format!("Failed to build request: {}", e)))?;

        let stream = self
            .client
            .chat()
            .create_stream(openai_request)
            .await
            .map_err(|e| Error::Provider(format!("OpenAI API error: {}", e)))?;

        let mapped_stream = stream.map(|result| match result {
            Ok(response) => {
                let chunk = extract_chunk(response);
                Ok(chunk)
            }
            Err(e) => Err(Error::Provider(format!("Stream error: {}", e))),
        });

        Ok(Box::pin(mapped_stream))
    }
}

fn extract_chunk(response: CreateChatCompletionStreamResponse) -> StreamChunk {
    let delta = response
        .choices
        .first()
        .and_then(|c| c.delta.content.as_ref())
        .map(|s| s.clone())
        .unwrap_or_default();

    let finish_reason = response
        .choices
        .first()
        .and_then(|c| c.finish_reason.as_ref())
        .map(|r| format!("{:?}", r));

    StreamChunk {
        delta,
        finish_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_provider_creation() {
        let config = OpenAIConfig {
            api_base: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-4".to_string(),
            max_retries: 3,
            timeout_seconds: 30,
        };

        let provider = OpenAIProvider::new("test-key".to_string(), config.clone());
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.config.default_model, "gpt-4");
    }

    #[test]
    fn test_message_conversion() {
        let config = OpenAIConfig {
            api_base: "https://api.openai.com/v1".to_string(),
            default_model: "gpt-4".to_string(),
            max_retries: 3,
            timeout_seconds: 30,
        };

        let provider = OpenAIProvider::new("test-key".to_string(), config);

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
        ];

        let converted = provider.convert_messages(messages);
        assert_eq!(converted.len(), 3);
    }

    #[test]
    fn test_extract_chunk() {
        // This would require mocking CreateChatCompletionStreamResponse
        // which is complex due to the async-openai types
        // For now, we'll focus on the integration tests
    }
}