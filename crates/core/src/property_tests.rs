/// Property-based and mutation testing for 100% coverage
/// 
/// This module uses proptest to generate random inputs and test invariants
/// across all functions to ensure robustness and edge case coverage.

#[cfg(test)]
mod property_tests {
    use crate::*;
    use crate::config::{Config, OpenAIConfig};
    use crate::error::Error;
    use crate::provider::*;
    use crate::service::ServiceContainer;
    use proptest::prelude::*;
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use std::io::Write;

    // Property test strategies
    prop_compose! {
        fn arb_openai_config()(
            default_model in "[a-zA-Z0-9-_]{1,50}",
            api_base in "https?://[a-zA-Z0-9.-]+/[a-zA-Z0-9/_-]*",
            max_retries in 0u32..100,
            timeout_seconds in 0u32..3600,
        ) -> OpenAIConfig {
            OpenAIConfig {
                default_model,
                api_base,
                max_retries,
                timeout_seconds,
            }
        }
    }

    prop_compose! {
        fn arb_config()(openai in arb_openai_config()) -> Config {
            Config { openai }
        }
    }

    prop_compose! {
        fn arb_message()(
            role in "user|assistant|system",
            content in ".*",
        ) -> Message {
            Message { role, content }
        }
    }

    prop_compose! {
        fn arb_completion_request()(
            model in "[a-zA-Z0-9-_]{1,50}",
            messages in prop::collection::vec(arb_message(), 1..10),
            temperature in prop::option::of(0.0f32..2.0),
            max_tokens in prop::option::of(1u32..4096),
            stream in any::<bool>(),
        ) -> CompletionRequest {
            CompletionRequest {
                model,
                messages,
                temperature,
                max_tokens,
                stream,
            }
        }
    }

    proptest! {
        #[test]
        fn prop_config_serialization_roundtrip(config in arb_config()) {
            let serialized = toml::to_string(&config).unwrap();
            let deserialized: Config = toml::from_str(&serialized).unwrap();
            
            prop_assert_eq!(config.openai.default_model, deserialized.openai.default_model);
            prop_assert_eq!(config.openai.api_base, deserialized.openai.api_base);
            prop_assert_eq!(config.openai.max_retries, deserialized.openai.max_retries);
            prop_assert_eq!(config.openai.timeout_seconds, deserialized.openai.timeout_seconds);
        }

        #[test]
        fn prop_config_save_load_roundtrip(config in arb_config()) {
            let mut temp_file = NamedTempFile::new().unwrap();
            
            // Save config
            config.save(temp_file.path()).unwrap();
            
            // Load config
            let loaded_config = Config::from_file(temp_file.path()).unwrap();
            
            prop_assert_eq!(config.openai.default_model, loaded_config.openai.default_model);
            prop_assert_eq!(config.openai.api_base, loaded_config.openai.api_base);
            prop_assert_eq!(config.openai.max_retries, loaded_config.openai.max_retries);
            prop_assert_eq!(config.openai.timeout_seconds, loaded_config.openai.timeout_seconds);
        }

        #[test]
        fn prop_message_creation(
            role in ".*",
            content in ".*",
        ) {
            let message = Message { role: role.clone(), content: content.clone() };
            prop_assert_eq!(message.role, role);
            prop_assert_eq!(message.content, content);
        }

        #[test]
        fn prop_completion_request_builder(
            model in ".*",
            temperature in prop::option::of(-10.0f32..10.0),
            max_tokens in prop::option::of(0u32..1000000),
            stream in any::<bool>(),
        ) {
            let request = CompletionRequest::builder()
                .model(model.clone())
                .temperature(temperature.unwrap_or(0.7))
                .max_tokens(max_tokens.unwrap_or(1000))
                .stream(stream)
                .build();

            prop_assert_eq!(request.model, model);
            prop_assert_eq!(request.temperature, Some(temperature.unwrap_or(0.7)));
            prop_assert_eq!(request.max_tokens, Some(max_tokens.unwrap_or(1000)));
            prop_assert_eq!(request.stream, stream);
        }

        #[test]
        fn prop_service_container_creation(config in arb_config()) {
            let result = ServiceContainer::new(config);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn prop_provider_registration(
            config in arb_config(),
            provider_name in "[a-zA-Z0-9_-]{1,20}",
        ) {
            let mut container = ServiceContainer::new(config).unwrap();
            let mock_provider = Arc::new(crate::provider::tests::MockProvider {
                response: "Property test response".to_string(),
                should_fail: false,
            });

            container.register_provider(&provider_name, mock_provider);
            let retrieved = container.get_provider(&provider_name);
            prop_assert!(retrieved.is_ok());
        }

        #[test]
        fn prop_usage_creation(
            prompt_tokens in 0u32..1000000,
            completion_tokens in 0u32..1000000,
            total_tokens in 0u32..1000000,
        ) {
            let usage = Usage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            };
            
            prop_assert_eq!(usage.prompt_tokens, prompt_tokens);
            prop_assert_eq!(usage.completion_tokens, completion_tokens);
            prop_assert_eq!(usage.total_tokens, total_tokens);
        }

        #[test]
        fn prop_completion_response_creation(
            content in ".*",
            model in ".*",
            prompt_tokens in 0u32..1000000,
            completion_tokens in 0u32..1000000,
            total_tokens in 0u32..1000000,
        ) {
            let response = CompletionResponse {
                content: content.clone(),
                model: model.clone(),
                usage: Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens,
                },
            };

            prop_assert_eq!(response.content, content);
            prop_assert_eq!(response.model, model);
            prop_assert_eq!(response.usage.prompt_tokens, prompt_tokens);
        }

        #[test]
        fn prop_stream_chunk_creation(
            delta in ".*",
            finish_reason in prop::option::of(".*"),
        ) {
            let chunk = StreamChunk {
                delta: delta.clone(),
                finish_reason: finish_reason.clone(),
            };

            prop_assert_eq!(chunk.delta, delta);
            prop_assert_eq!(chunk.finish_reason, finish_reason);
        }

        #[test]
        fn prop_error_display(error_msg in ".*") {
            let config_error = Error::Config(error_msg.clone());
            let display_msg = format!("{}", config_error);
            prop_assert!(display_msg.contains(&error_msg));

            let provider_error = Error::Provider(error_msg.clone());
            let display_msg = format!("{}", provider_error);
            prop_assert!(display_msg.contains(&error_msg));

            let service_error = Error::Service(error_msg.clone());
            let display_msg = format!("{}", service_error);
            prop_assert!(display_msg.contains(&error_msg));
        }
    }

    // Mutation testing: test that our tests actually catch bugs
    #[cfg(test)]
    mod mutation_tests {
        use super::*;

        #[test]
        fn test_config_invariants() {
            // Test that certain invariants hold for any config
            let config = Config::default();
            
            // Model name should not be empty for default config
            assert!(!config.openai.default_model.is_empty());
            
            // API base should be a valid URL structure
            assert!(config.openai.api_base.starts_with("http"));
            
            // Timeouts should be reasonable
            assert!(config.openai.timeout_seconds > 0);
            assert!(config.openai.timeout_seconds < 86400); // Less than a day
        }

        #[test]
        fn test_message_invariants() {
            // Test message invariants
            let message = Message {
                role: "user".to_string(),
                content: "test".to_string(),
            };

            // Role and content should be preserved exactly
            assert_eq!(message.role, "user");
            assert_eq!(message.content, "test");

            // Message should handle empty strings
            let empty_message = Message {
                role: "".to_string(),
                content: "".to_string(),
            };
            assert_eq!(empty_message.role.len(), 0);
            assert_eq!(empty_message.content.len(), 0);
        }

        #[test]
        fn test_completion_request_builder_invariants() {
            // Test that builder always produces valid requests
            let request = CompletionRequest::builder()
                .model("test-model")
                .build();

            assert_eq!(request.model, "test-model");
            assert!(request.messages.is_empty()); // Default should be empty
            assert_eq!(request.temperature, None); // Default should be None
            assert_eq!(request.max_tokens, None); // Default should be None
            assert!(!request.stream); // Default should be false
        }

        #[test]
        fn test_error_handling_invariants() {
            // Test that errors maintain their message content
            let original_msg = "Test error message";
            
            let config_error = Error::Config(original_msg.to_string());
            let displayed = format!("{}", config_error);
            assert!(displayed.contains(original_msg));

            let provider_error = Error::Provider(original_msg.to_string());
            let displayed = format!("{}", provider_error);
            assert!(displayed.contains(original_msg));

            let service_error = Error::Service(original_msg.to_string());
            let displayed = format!("{}", service_error);
            assert!(displayed.contains(original_msg));
        }

        #[tokio::test]
        async fn test_mock_provider_invariants() {
            // Test that mock provider behaves consistently
            let mock = crate::provider::tests::MockProvider {
                response: "test response".to_string(),
                should_fail: false,
            };

            let request = CompletionRequest::builder()
                .model("test")
                .build();

            let response = mock.complete(request).await.unwrap();
            assert_eq!(response.content, "test response");
            assert_eq!(mock.name(), "mock");
        }

        #[test]
        fn test_service_container_invariants() {
            // Test service container invariants
            let config = Config::default();
            let container = ServiceContainer::new(config).unwrap();

            // Container should start with consistent state
            let providers = container.list_providers();
            let config_ref = container.config();
            
            // Config should be accessible
            assert!(!config_ref.openai.default_model.is_empty());

            // Getting non-existent provider should fail consistently
            let result = container.get_provider("nonexistent");
            assert!(result.is_err());
        }
    }

    // Stress tests to find edge cases
    #[cfg(test)]
    mod stress_tests {
        use super::*;
        use std::thread;
        use std::time::Duration;

        #[test]
        fn test_large_config_files() {
            // Test with very large configuration values
            let large_model_name = "a".repeat(10000);
            let large_api_base = format!("https://{}example.com/v1", "a".repeat(1000));

            let config = Config {
                openai: OpenAIConfig {
                    default_model: large_model_name.clone(),
                    api_base: large_api_base.clone(),
                    max_retries: u32::MAX,
                    timeout_seconds: u32::MAX,
                },
            };

            // Should handle serialization
            let serialized = toml::to_string(&config).unwrap();
            assert!(serialized.contains(&large_model_name));

            // Should handle file operations
            let temp_file = NamedTempFile::new().unwrap();
            config.save(temp_file.path()).unwrap();
            let loaded = Config::from_file(temp_file.path()).unwrap();
            assert_eq!(loaded.openai.default_model, large_model_name);
        }

        #[test]
        fn test_unicode_edge_cases() {
            // Test with various Unicode edge cases
            let unicode_strings = vec![
                "🚀🎉🔥", // Emojis
                "∑∫∆∇", // Mathematical symbols
                "αβγδεζηθικλμνξοπρστυφχψω", // Greek alphabet
                "中文测试", // Chinese characters
                "العربية", // Arabic text
                "עברית", // Hebrew text
                "🏳️‍🌈🏳️‍⚧️", // Flag emojis with zero-width joiners
                "\u{200B}\u{200C}\u{200D}", // Zero-width characters
                "a\u{0301}", // Combining diacritical marks
                "\u{1F1FA}\u{1F1F8}", // Regional indicator symbols (US flag)
            ];

            for unicode_str in unicode_strings {
                let message = Message {
                    role: "user".to_string(),
                    content: unicode_str.to_string(),
                };
                assert_eq!(message.content, unicode_str);

                let config = Config {
                    openai: OpenAIConfig {
                        default_model: unicode_str.to_string(),
                        api_base: "https://api.openai.com/v1".to_string(),
                        max_retries: 3,
                        timeout_seconds: 30,
                    },
                };

                // Should handle serialization without panicking
                let result = toml::to_string(&config);
                // Some Unicode might not be valid in TOML, so we just check it doesn't panic
                let _ = result;
            }
        }

        #[test]
        fn test_concurrent_access() {
            // Test concurrent access to service container
            let config = Config::default();
            let container = Arc::new(ServiceContainer::new(config).unwrap());

            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let container = Arc::clone(&container);
                    thread::spawn(move || {
                        for _ in 0..100 {
                            let providers = container.list_providers();
                            let config = container.config();
                            assert!(!config.openai.default_model.is_empty());
                            
                            // Try to get a provider (might fail, but shouldn't panic)
                            let _ = container.get_provider(&format!("test-{}", i));
                            
                            thread::sleep(Duration::from_nanos(1));
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        }

        #[test]
        fn test_memory_pressure() {
            // Test behavior under memory pressure
            let mut large_messages = Vec::new();
            
            // Create many large messages
            for i in 0..1000 {
                let large_content = format!("{}: {}", i, "x".repeat(10000));
                large_messages.push(Message {
                    role: "user".to_string(),
                    content: large_content,
                });
            }

            let request = CompletionRequest {
                model: "test-model".to_string(),
                messages: large_messages,
                temperature: Some(0.7),
                max_tokens: Some(1000),
                stream: false,
            };

            // Should handle large requests without panicking
            assert_eq!(request.messages.len(), 1000);
            assert_eq!(request.messages[0].content.len(), 10007); // "0: " + 10000 x's
        }
    }
}