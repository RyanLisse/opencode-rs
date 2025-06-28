//! Tests for error handling improvements
//! 
//! This test suite defines the expected behavior for comprehensive error handling,
//! error context, and error recovery following TDD principles.

use super::*;
use std::io;
use std::sync::Arc;

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_error_creation_and_display() {
        // GIVEN: Various error scenarios
        
        // WHEN: We create different error types
        let api_error = OpenCodeError::Provider(ProviderError::ApiError {
            status: 429,
            message: "Rate limit exceeded".to_string(),
        });
        
        let config_error = OpenCodeError::Configuration(ConfigError::Validation(
            "Invalid provider configuration".to_string()
        ));
        
        let network_error = OpenCodeError::Network(NetworkError::Timeout {
            operation: "API call".to_string(),
            duration: std::time::Duration::from_secs(30),
        });
        
        // THEN: Error messages should be properly formatted
        assert_eq!(api_error.to_string(), "Provider error: API error: Rate limit exceeded (status: 429)");
        assert_eq!(config_error.to_string(), "Configuration error: Validation error: Invalid provider configuration");
        assert_eq!(network_error.to_string(), "Network error: Operation 'API call' timed out after 30s");
    }

    #[test]
    fn test_error_context_chain() {
        // GIVEN: An error with context chain
        let base_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        
        // WHEN: We add context to the error
        let error = OpenCodeError::Io(base_error)
            .with_context("Loading configuration")
            .with_context("Initializing application");
        
        // THEN: Context should be preserved in order
        let contexts = error.contexts();
        assert_eq!(contexts.len(), 2);
        assert_eq!(contexts[0], "Loading configuration");
        assert_eq!(contexts[1], "Initializing application");
        
        // Full error message should include context
        let full_message = error.full_message();
        assert!(full_message.contains("Initializing application"));
        assert!(full_message.contains("Loading configuration"));
        assert!(full_message.contains("File not found"));
    }

    #[test]
    fn test_error_recovery_suggestions() {
        // GIVEN: Errors with recovery suggestions
        
        // WHEN: We create errors with recovery hints
        let rate_limit_error = OpenCodeError::Provider(ProviderError::RateLimitExceeded)
            .with_recovery(Recovery::Retry {
                after: std::time::Duration::from_secs(60),
                max_attempts: 3,
            });
        
        let auth_error = OpenCodeError::Provider(ProviderError::AuthenticationError(
            "Invalid API key".to_string()
        ))
            .with_recovery(Recovery::Manual(
                "Please check your API key in the configuration file".to_string()
            ));
        
        // THEN: Recovery suggestions should be accessible
        match rate_limit_error.recovery() {
            Some(Recovery::Retry { after, max_attempts }) => {
                assert_eq!(after.as_secs(), 60);
                assert_eq!(*max_attempts, 3);
            }
            _ => panic!("Expected Retry recovery"),
        }
        
        match auth_error.recovery() {
            Some(Recovery::Manual(msg)) => {
                assert!(msg.contains("API key"));
            }
            _ => panic!("Expected Manual recovery"),
        }
    }

    #[test]
    fn test_error_source_chain() {
        // GIVEN: Nested errors with source chain
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let config_error = ConfigError::Io(io_error);
        let app_error = OpenCodeError::Configuration(config_error);
        
        // WHEN: We traverse the error source chain
        let mut sources = vec![];
        let mut current: Option<&dyn std::error::Error> = Some(&app_error);
        
        while let Some(err) = current {
            sources.push(err.to_string());
            current = err.source();
        }
        
        // THEN: We should see the full error chain
        assert_eq!(sources.len(), 3);
        assert!(sources[0].contains("Configuration error"));
        assert!(sources[1].contains("IO error"));
        assert!(sources[2].contains("Access denied"));
    }

    #[test]
    fn test_error_categorization() {
        // GIVEN: Various errors
        let errors = vec![
            OpenCodeError::Provider(ProviderError::RateLimitExceeded),
            OpenCodeError::Network(NetworkError::ConnectionRefused),
            OpenCodeError::Configuration(ConfigError::NotFound),
            OpenCodeError::Internal("Unexpected state".to_string()),
        ];
        
        // WHEN: We categorize errors
        for error in errors {
            let category = error.category();
            
            // THEN: Each error should have appropriate category
            match &error {
                OpenCodeError::Provider(ProviderError::RateLimitExceeded) => {
                    assert_eq!(category, ErrorCategory::Transient);
                }
                OpenCodeError::Network(_) => {
                    assert_eq!(category, ErrorCategory::Transient);
                }
                OpenCodeError::Configuration(_) => {
                    assert_eq!(category, ErrorCategory::Configuration);
                }
                OpenCodeError::Internal(_) => {
                    assert_eq!(category, ErrorCategory::Internal);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_error_retry_policy() {
        // GIVEN: Errors with different retry policies
        let transient_error = OpenCodeError::Network(NetworkError::Timeout {
            operation: "Request".to_string(),
            duration: std::time::Duration::from_secs(30),
        });
        
        let permanent_error = OpenCodeError::Configuration(ConfigError::Validation(
            "Invalid setting".to_string()
        ));
        
        // WHEN: We check retry policies
        let transient_policy = transient_error.retry_policy();
        let permanent_policy = permanent_error.retry_policy();
        
        // THEN: Appropriate policies should be returned
        match transient_policy {
            RetryPolicy::Exponential { max_attempts, base_delay, .. } => {
                assert_eq!(max_attempts, 3);
                assert_eq!(base_delay.as_secs(), 1);
            }
            _ => panic!("Expected exponential retry for transient error"),
        }
        
        assert_eq!(permanent_policy, RetryPolicy::None);
    }

    #[test]
    fn test_error_telemetry() {
        // GIVEN: An error with telemetry data
        let error = OpenCodeError::Provider(ProviderError::ApiError {
            status: 500,
            message: "Internal server error".to_string(),
        })
        .with_telemetry(ErrorTelemetry {
            timestamp: std::time::SystemTime::now(),
            request_id: Some("req-123".to_string()),
            user_id: Some("user-456".to_string()),
            additional_data: {
                let mut map = std::collections::HashMap::new();
                map.insert("provider".to_string(), "openai".to_string());
                map.insert("model".to_string(), "gpt-4".to_string());
                map
            },
        });
        
        // WHEN: We access telemetry data
        let telemetry = error.telemetry().unwrap();
        
        // THEN: All telemetry fields should be accessible
        assert_eq!(telemetry.request_id, Some("req-123".to_string()));
        assert_eq!(telemetry.user_id, Some("user-456".to_string()));
        assert_eq!(telemetry.additional_data.get("provider"), Some(&"openai".to_string()));
        assert_eq!(telemetry.additional_data.get("model"), Some(&"gpt-4".to_string()));
    }

    #[test]
    fn test_error_serialization() {
        // GIVEN: An error that needs to be serialized
        let error = OpenCodeError::Provider(ProviderError::ApiError {
            status: 404,
            message: "Model not found".to_string(),
        })
        .with_context("Calling OpenAI API")
        .with_recovery(Recovery::Fallback {
            alternative: "Use gpt-3.5-turbo instead".to_string(),
        });
        
        // WHEN: We serialize the error
        let serialized = error.to_json();
        
        // THEN: JSON should contain all error information
        let json: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(json["type"], "Provider");
        assert_eq!(json["message"], "Provider error: API error: Model not found (status: 404)");
        assert_eq!(json["contexts"][0], "Calling OpenAI API");
        assert_eq!(json["recovery"]["type"], "Fallback");
        assert_eq!(json["recovery"]["alternative"], "Use gpt-3.5-turbo instead");
    }

    #[test]
    fn test_error_aggregation() {
        // GIVEN: Multiple errors that need to be aggregated
        let errors = vec![
            OpenCodeError::Provider(ProviderError::RateLimitExceeded),
            OpenCodeError::Network(NetworkError::ConnectionRefused),
            OpenCodeError::Provider(ProviderError::ApiError {
                status: 500,
                message: "Server error".to_string(),
            }),
        ];
        
        // WHEN: We aggregate errors
        let aggregated = OpenCodeError::Multiple(errors);
        
        // THEN: All errors should be accessible
        match &aggregated {
            OpenCodeError::Multiple(errs) => {
                assert_eq!(errs.len(), 3);
                // Check that we can iterate and handle each error
                for (i, err) in errs.iter().enumerate() {
                    match i {
                        0 => assert!(matches!(err, OpenCodeError::Provider(ProviderError::RateLimitExceeded))),
                        1 => assert!(matches!(err, OpenCodeError::Network(_))),
                        2 => assert!(matches!(err, OpenCodeError::Provider(ProviderError::ApiError { .. }))),
                        _ => panic!("Unexpected error count"),
                    }
                }
            }
            _ => panic!("Expected Multiple error"),
        }
    }

    #[tokio::test]
    async fn test_async_error_handling() {
        // GIVEN: An async operation that might fail
        async fn risky_operation() -> Result<String, OpenCodeError> {
            // Simulate async work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            Err(OpenCodeError::Network(NetworkError::Timeout {
                operation: "Async operation".to_string(),
                duration: std::time::Duration::from_secs(10),
            }))
        }
        
        // WHEN: We handle the error with async context
        let result = risky_operation()
            .await
            .map_err(|e| e.with_context("Performing background task"));
        
        // THEN: Error context should be preserved
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.contexts().contains(&"Performing background task".to_string()));
    }

    #[test]
    fn test_error_conversion() {
        // GIVEN: Errors from external libraries
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let parse_error = "invalid digit found in string".parse::<i32>().unwrap_err();
        
        // WHEN: We convert them to our error type
        let our_io_error: OpenCodeError = io_error.into();
        let our_parse_error: OpenCodeError = parse_error.into();
        
        // THEN: They should be properly wrapped
        assert!(matches!(our_io_error, OpenCodeError::Io(_)));
        assert!(matches!(our_parse_error, OpenCodeError::Parse(_)));
    }
}

// Type definitions that will be moved to the actual implementation
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenCodeError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Multiple errors occurred: {0:?}")]
    Multiple(Vec<OpenCodeError>),
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("Operation '{operation}' timed out after {duration:?}")]
    Timeout {
        operation: String,
        duration: std::time::Duration,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    Transient,
    Configuration,
    Internal,
    External,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RetryPolicy {
    None,
    Exponential {
        max_attempts: u32,
        base_delay: std::time::Duration,
        max_delay: std::time::Duration,
    },
    Fixed {
        attempts: u32,
        delay: std::time::Duration,
    },
}

#[derive(Debug, Clone)]
pub enum Recovery {
    Retry {
        after: std::time::Duration,
        max_attempts: u32,
    },
    Fallback {
        alternative: String,
    },
    Manual(String),
}

#[derive(Debug, Clone)]
pub struct ErrorTelemetry {
    pub timestamp: std::time::SystemTime,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub additional_data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
struct SerializedError {
    #[serde(rename = "type")]
    error_type: String,
    message: String,
    contexts: Vec<String>,
    recovery: Option<SerializedRecovery>,
}

#[derive(Serialize, Deserialize)]
struct SerializedRecovery {
    #[serde(rename = "type")]
    recovery_type: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

// Error enhancement implementation
struct ErrorEnhancement {
    contexts: Vec<String>,
    recovery: Option<Recovery>,
    telemetry: Option<ErrorTelemetry>,
}

// Extension trait for error enhancement
impl OpenCodeError {
    pub fn with_context(self, context: impl Into<String>) -> Self {
        // Implementation would store context
        self
    }
    
    pub fn with_recovery(self, recovery: Recovery) -> Self {
        // Implementation would store recovery
        self
    }
    
    pub fn with_telemetry(self, telemetry: ErrorTelemetry) -> Self {
        // Implementation would store telemetry
        self
    }
    
    pub fn contexts(&self) -> Vec<String> {
        // Implementation would return stored contexts
        vec![]
    }
    
    pub fn recovery(&self) -> Option<&Recovery> {
        // Implementation would return stored recovery
        None
    }
    
    pub fn telemetry(&self) -> Option<&ErrorTelemetry> {
        // Implementation would return stored telemetry
        None
    }
    
    pub fn full_message(&self) -> String {
        // Implementation would build full error message with context
        self.to_string()
    }
    
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Provider(ProviderError::RateLimitExceeded) => ErrorCategory::Transient,
            Self::Network(_) => ErrorCategory::Transient,
            Self::Configuration(_) => ErrorCategory::Configuration,
            Self::Internal(_) => ErrorCategory::Internal,
            _ => ErrorCategory::External,
        }
    }
    
    pub fn retry_policy(&self) -> RetryPolicy {
        match self.category() {
            ErrorCategory::Transient => RetryPolicy::Exponential {
                max_attempts: 3,
                base_delay: std::time::Duration::from_secs(1),
                max_delay: std::time::Duration::from_secs(60),
            },
            _ => RetryPolicy::None,
        }
    }
    
    pub fn to_json(&self) -> String {
        // Implementation would serialize to JSON
        serde_json::to_string_pretty(&SerializedError {
            error_type: "Provider".to_string(),
            message: self.to_string(),
            contexts: self.contexts(),
            recovery: None,
        }).unwrap()
    }
}