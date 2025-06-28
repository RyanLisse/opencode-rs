//! Tests for the Provider abstraction trait
//! 
//! This test suite defines the expected behavior for AI provider implementations
//! following TDD principles.

use super::*;
use async_trait::async_trait;
use mockall::*;

#[cfg(test)]
mod provider_trait_tests {
    use super::*;

    #[test]
    fn test_provider_trait_requirements() {
        // Verify that Provider trait has all required methods
        fn assert_provider_trait<T: Provider>() {}
        
        // This test ensures the trait has the right shape
        // Compilation will fail if trait requirements change
    }

    #[tokio::test]
    async fn test_provider_completion() {
        // GIVEN: A mock provider implementation
        let mut mock_provider = MockProvider::new();
        
        // WHEN: We set up expectations for a completion request
        mock_provider
            .expect_complete()
            .with(mockall::predicate::function(|req: &CompletionRequest| {
                req.messages.len() == 1 && 
                req.messages[0].role == MessageRole::User &&
                req.messages[0].content == "Hello, AI!"
            }))
            .times(1)
            .returning(|_| {
                Ok(CompletionResponse {
                    id: "test-123".to_string(),
                    model: "test-model".to_string(),
                    choices: vec![
                        Choice {
                            message: Message {
                                role: MessageRole::Assistant,
                                content: "Hello! How can I help you?".to_string(),
                            },
                            finish_reason: FinishReason::Stop,
                            index: 0,
                        }
                    ],
                    usage: Usage {
                        prompt_tokens: 10,
                        completion_tokens: 8,
                        total_tokens: 18,
                    },
                    created: 1234567890,
                })
            });

        // THEN: The provider should return the expected response
        let request = CompletionRequest {
            model: "test-model".to_string(),
            messages: vec![
                Message {
                    role: MessageRole::User,
                    content: "Hello, AI!".to_string(),
                }
            ],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: false,
        };

        let response = mock_provider.complete(request).await.unwrap();
        assert_eq!(response.id, "test-123");
        assert_eq!(response.choices.len(), 1);
        assert_eq!(response.choices[0].message.content, "Hello! How can I help you?");
    }

    #[tokio::test]
    async fn test_provider_streaming() {
        // GIVEN: A mock provider that supports streaming
        let mut mock_provider = MockProvider::new();
        
        // WHEN: We request a streaming completion
        mock_provider
            .expect_complete_stream()
            .times(1)
            .returning(|_| {
                let (tx, rx) = tokio::sync::mpsc::channel(10);
                
                tokio::spawn(async move {
                    // Simulate streaming chunks
                    for chunk in ["Hello", " from", " streaming", " AI!"] {
                        let _ = tx.send(Ok(StreamChunk {
                            id: "stream-123".to_string(),
                            choices: vec![
                                StreamChoice {
                                    delta: Delta {
                                        content: Some(chunk.to_string()),
                                        role: None,
                                    },
                                    index: 0,
                                    finish_reason: None,
                                }
                            ],
                            created: 1234567890,
                        })).await;
                    }
                });
                
                Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
            });

        // THEN: We should receive all streaming chunks
        let request = CompletionRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Stream this!".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: true,
        };

        let mut stream = mock_provider.complete_stream(request).await.unwrap();
        let mut full_response = String::new();
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            if let Some(content) = &chunk.choices[0].delta.content {
                full_response.push_str(content);
            }
        }
        
        assert_eq!(full_response, "Hello from streaming AI!");
    }

    #[tokio::test]
    async fn test_provider_error_handling() {
        // GIVEN: A mock provider that returns errors
        let mut mock_provider = MockProvider::new();
        
        // WHEN: The provider encounters an API error
        mock_provider
            .expect_complete()
            .times(1)
            .returning(|_| {
                Err(ProviderError::ApiError {
                    status: 429,
                    message: "Rate limit exceeded".to_string(),
                })
            });

        // THEN: The error should be properly propagated
        let request = CompletionRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: MessageRole::User,
                content: "Test".to_string(),
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
        };

        let result = mock_provider.complete(request).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ProviderError::ApiError { status, message } => {
                assert_eq!(status, 429);
                assert_eq!(message, "Rate limit exceeded");
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn test_provider_capabilities() {
        // GIVEN: Different provider implementations
        let mut mock_provider = MockProvider::new();
        
        // WHEN: We query provider capabilities
        mock_provider
            .expect_capabilities()
            .times(1)
            .returning(|| {
                ProviderCapabilities {
                    supports_streaming: true,
                    supports_function_calling: true,
                    supports_vision: false,
                    max_tokens: 4096,
                    models: vec![
                        ModelInfo {
                            id: "gpt-4".to_string(),
                            display_name: "GPT-4".to_string(),
                            context_window: 8192,
                            max_output_tokens: 4096,
                        }
                    ],
                }
            });

        // THEN: Capabilities should be correctly reported
        let caps = mock_provider.capabilities();
        assert!(caps.supports_streaming);
        assert!(caps.supports_function_calling);
        assert!(!caps.supports_vision);
        assert_eq!(caps.max_tokens, 4096);
        assert_eq!(caps.models.len(), 1);
    }

    #[test]
    fn test_provider_configuration() {
        // Test that providers can be configured with different settings
        // This will be implemented based on the configuration trait
    }
}

// Mock implementations for testing
#[cfg(test)]
mockall::mock! {
    Provider {}
    
    #[async_trait]
    impl Provider for Provider {
        async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError>;
        async fn complete_stream(&self, request: CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>;
        fn capabilities(&self) -> ProviderCapabilities;
        fn name(&self) -> &str;
    }
}

// Type definitions that will be moved to the actual implementation
#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub created: i64,
}

#[derive(Debug, Clone)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: FinishReason,
    pub index: u32,
}

#[derive(Debug, Clone)]
pub enum FinishReason {
    Stop,
    Length,
    FunctionCall,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct StreamChunk {
    pub id: String,
    pub choices: Vec<StreamChoice>,
    pub created: i64,
}

#[derive(Debug, Clone)]
pub struct StreamChoice {
    pub delta: Delta,
    pub index: u32,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Debug, Clone)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<MessageRole>,
}

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub supports_vision: bool,
    pub max_tokens: u32,
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub context_window: u32,
    pub max_output_tokens: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("API error: {message} (status: {status})")]
    ApiError { status: u16, message: String },
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, ProviderError>;
    async fn complete_stream(&self, request: CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, ProviderError>> + Send>>, ProviderError>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn name(&self) -> &str;
}