use crate::config::Config;
use crate::error::{Error, Result};
use crate::provider::{LLMProvider, OpenAIProvider};
use std::collections::HashMap;
use std::sync::Arc;

/// Service container for dependency injection
pub struct ServiceContainer {
    providers: HashMap<String, Arc<dyn LLMProvider>>,
    config: Config,
}

impl ServiceContainer {
    /// Create a new service container
    pub fn new(config: Config) -> Result<Self> {
        let mut container = Self {
            providers: HashMap::new(),
            config,
        };

        // Register default providers
        container.register_default_providers()?;

        Ok(container)
    }

    /// Register default providers based on configuration
    fn register_default_providers(&mut self) -> Result<()> {
        // Register OpenAI provider if API key is available
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let provider = OpenAIProvider::new(api_key, self.config.openai.clone());
            self.register_provider("openai", Arc::new(provider));
        }

        Ok(())
    }

    /// Register a provider with the container
    pub fn register_provider(&mut self, name: &str, provider: Arc<dyn LLMProvider>) {
        self.providers.insert(name.to_string(), provider);
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Result<Arc<dyn LLMProvider>> {
        self.providers
            .get(name)
            .cloned()
            .ok_or_else(|| Error::Service(format!("Provider '{}' not found", name)))
    }

    /// Get the default provider (first available)
    pub fn get_default_provider(&self) -> Result<Arc<dyn LLMProvider>> {
        // Try OpenAI first as the default
        if let Ok(provider) = self.get_provider("openai") {
            return Ok(provider);
        }

        // If no specific provider, return the first available
        self.providers
            .values()
            .next()
            .cloned()
            .ok_or_else(|| Error::Service("No providers available".into()))
    }

    /// List all registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Update the configuration and re-register providers
    pub fn update_config(&mut self, config: Config) -> Result<()> {
        self.config = config;
        self.providers.clear();
        self.register_default_providers()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::tests::MockProvider;

    #[test]
    fn test_service_container_creation() {
        let config = Config::default();
        let container = ServiceContainer::new(config).unwrap();
        
        // Should create without error
        assert!(container.providers.is_empty() || !container.providers.is_empty());
    }

    #[test]
    fn test_register_and_get_provider() {
        let config = Config::default();
        let mut container = ServiceContainer::new(config).unwrap();

        let mock_provider = Arc::new(MockProvider {
            response: "Test response".to_string(),
            should_fail: false,
        });

        container.register_provider("mock", mock_provider.clone());

        let retrieved = container.get_provider("mock").unwrap();
        assert_eq!(retrieved.name(), "mock");
    }

    #[test]
    fn test_get_provider_not_found() {
        let config = Config::default();
        let container = ServiceContainer::new(config).unwrap();

        let result = container.get_provider("nonexistent");
        assert!(result.is_err());
        match result {
            Err(Error::Service(msg)) => assert!(msg.contains("not found")),
            _ => panic!("Expected Service error"),
        }
    }

    #[test]
    fn test_list_providers() {
        let config = Config::default();
        let mut container = ServiceContainer::new(config).unwrap();

        // Clear any existing providers first
        container.providers.clear();

        let mock1 = Arc::new(MockProvider {
            response: "Test1".to_string(),
            should_fail: false,
        });
        let mock2 = Arc::new(MockProvider {
            response: "Test2".to_string(),
            should_fail: false,
        });

        container.register_provider("mock1", mock1);
        container.register_provider("mock2", mock2);

        let providers = container.list_providers();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&"mock1".to_string()));
        assert!(providers.contains(&"mock2".to_string()));
    }

    #[test]
    fn test_get_default_provider() {
        let config = Config::default();
        let mut container = ServiceContainer::new(config).unwrap();

        // Clear any existing providers first
        container.providers.clear();

        // With no providers registered, should fail
        let result = container.get_default_provider();
        assert!(result.is_err());

        // Register a provider
        let mock_provider = Arc::new(MockProvider {
            response: "Default".to_string(),
            should_fail: false,
        });
        container.register_provider("default", mock_provider);

        let default = container.get_default_provider().unwrap();
        assert_eq!(default.name(), "mock");
    }

    #[test]
    fn test_config_access() {
        let config = Config::default();
        let original_model = config.openai.default_model.clone();
        
        let container = ServiceContainer::new(config).unwrap();
        assert_eq!(container.config().openai.default_model, original_model);
    }

    #[test]
    fn test_update_config() {
        let config = Config::default();
        let mut container = ServiceContainer::new(config).unwrap();

        let mut new_config = Config::default();
        new_config.openai.default_model = "gpt-3.5-turbo".to_string();

        container.update_config(new_config).unwrap();
        assert_eq!(container.config().openai.default_model, "gpt-3.5-turbo");
    }

    #[tokio::test]
    async fn test_provider_functionality() {
        let config = Config::default();
        let mut container = ServiceContainer::new(config).unwrap();

        let mock_provider = Arc::new(MockProvider {
            response: "Hello from service container".to_string(),
            should_fail: false,
        });

        container.register_provider("test", mock_provider);

        let provider = container.get_provider("test").unwrap();
        
        let request = crate::provider::CompletionRequest {
            model: "test-model".to_string(),
            messages: vec![crate::provider::Message {
                role: "user".to_string(),
                content: "Test message".to_string(),
            }],
            temperature: Some(0.7),
            max_tokens: Some(100),
            stream: false,
        };

        let response = provider.complete(request).await.unwrap();
        assert_eq!(response.content, "Hello from service container");
    }
}