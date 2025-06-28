use super::*;
use async_trait::async_trait;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct MockProvider {
    pub response: String,
    pub should_fail: bool,
}

#[async_trait]
impl LLMProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        if self.should_fail {
            return Err(Error::Provider("Mock provider error".into()));
        }

        Ok(CompletionResponse {
            content: self.response.clone(),
            model: request.model,
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            },
        })
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<BoxStream<'static, Result<StreamChunk>>> {
        if self.should_fail {
            return Err(Error::Provider("Mock provider error".into()));
        }

        let chunks = vec![
            StreamChunk {
                delta: self.response.clone(),
                finish_reason: None,
            },
            StreamChunk {
                delta: String::new(),
                finish_reason: Some("stop".to_string()),
            },
        ];

        Ok(Box::pin(tokio_stream::iter(chunks.into_iter().map(Ok))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_complete() {
        let provider = MockProvider {
            response: "Test response".to_string(),
            should_fail: false,
        };

        let request = CompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: false,
        };

        let response = provider.complete(request.clone()).await.unwrap();
        assert_eq!(response.content, "Test response");
        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.usage.total_tokens, 30);
    }

    #[tokio::test]
    async fn test_mock_provider_complete_error() {
        let provider = MockProvider {
            response: String::new(),
            should_fail: true,
        };

        let request = CompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());
        match result {
            Err(Error::Provider(msg)) => assert_eq!(msg, "Mock provider error"),
            _ => panic!("Expected Provider error"),
        }
    }

    #[tokio::test]
    async fn test_mock_provider_stream() {
        let provider = MockProvider {
            response: "Streaming response".to_string(),
            should_fail: false,
        };

        let request = CompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![Message {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            }],
            temperature: Some(0.5),
            max_tokens: Some(200),
            stream: true,
        };

        let mut stream = provider.stream(request).await.unwrap();
        
        let mut chunks = Vec::new();
        while let Some(chunk) = stream.next().await {
            chunks.push(chunk.unwrap());
        }

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].delta, "Streaming response");
        assert_eq!(chunks[0].finish_reason, None);
        assert_eq!(chunks[1].delta, "");
        assert_eq!(chunks[1].finish_reason, Some("stop".to_string()));
    }

    #[tokio::test]
    async fn test_provider_trait_methods() {
        let provider = MockProvider {
            response: "Test".to_string(),
            should_fail: false,
        };

        assert_eq!(provider.name(), "mock");
    }

    #[test]
    fn test_message_construction() {
        let msg = Message {
            role: "assistant".to_string(),
            content: "I can help with that".to_string(),
        };

        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, "I can help with that");
    }

    #[test]
    fn test_completion_request_builder() {
        let request = CompletionRequest {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a coding assistant".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Write a hello world program".to_string(),
                },
            ],
            temperature: Some(0.8),
            max_tokens: Some(1000),
            stream: true,
        };

        assert_eq!(request.model, "gpt-3.5-turbo");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.temperature, Some(0.8));
        assert_eq!(request.max_tokens, Some(1000));
        assert!(request.stream);
    }

    #[test]
    fn test_usage_calculation() {
        let usage = Usage {
            prompt_tokens: 50,
            completion_tokens: 100,
            total_tokens: 150,
        };

        assert_eq!(usage.prompt_tokens, 50);
        assert_eq!(usage.completion_tokens, 100);
        assert_eq!(usage.total_tokens, 150);
    }
}