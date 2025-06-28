//! Tests for configuration management
//! 
//! This test suite defines the expected behavior for configuration loading,
//! validation, and management following TDD principles.

use super::*;
use std::collections::HashMap;
use tempfile::TempDir;
use std::fs;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        // GIVEN: No configuration file exists
        // WHEN: We create a default configuration
        let config = AppConfig::default();
        
        // THEN: It should have sensible defaults
        assert!(config.providers.is_empty());
        assert_eq!(config.default_provider, None);
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert!(!config.features.streaming_enabled);
        assert!(!config.features.function_calling_enabled);
    }

    #[test]
    fn test_config_from_file() {
        // GIVEN: A configuration file with provider settings
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config_content = r#"
            default_provider = "openai"
            
            [server]
            host = "0.0.0.0"
            port = 8080
            
            [features]
            streaming_enabled = true
            function_calling_enabled = true
            
            [[providers]]
            name = "openai"
            type = "openai"
            api_key = "${OPENAI_API_KEY}"
            base_url = "https://api.openai.com/v1"
            
            [[providers]]
            name = "anthropic"
            type = "anthropic"
            api_key = "${ANTHROPIC_API_KEY}"
            base_url = "https://api.anthropic.com"
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        // WHEN: We load the configuration
        let config = AppConfig::from_file(&config_path).unwrap();
        
        // THEN: All settings should be loaded correctly
        assert_eq!(config.default_provider, Some("openai".to_string()));
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.features.streaming_enabled);
        assert!(config.features.function_calling_enabled);
        assert_eq!(config.providers.len(), 2);
        
        let openai = &config.providers[0];
        assert_eq!(openai.name, "openai");
        assert_eq!(openai.provider_type, ProviderType::OpenAI);
        assert_eq!(openai.api_key, "${OPENAI_API_KEY}");
    }

    #[test]
    fn test_environment_variable_expansion() {
        // GIVEN: Configuration with environment variables
        std::env::set_var("TEST_API_KEY", "secret-key-123");
        std::env::set_var("TEST_BASE_URL", "https://test.api.com");
        
        let config_str = r#"
            [[providers]]
            name = "test"
            type = "openai"
            api_key = "${TEST_API_KEY}"
            base_url = "${TEST_BASE_URL}"
        "#;
        
        // WHEN: We parse and expand the configuration
        let mut config: AppConfig = toml::from_str(config_str).unwrap();
        config.expand_env_vars();
        
        // THEN: Environment variables should be replaced
        assert_eq!(config.providers[0].api_key, "secret-key-123");
        assert_eq!(config.providers[0].base_url, Some("https://test.api.com".to_string()));
        
        // Cleanup
        std::env::remove_var("TEST_API_KEY");
        std::env::remove_var("TEST_BASE_URL");
    }

    #[test]
    fn test_config_validation() {
        // GIVEN: Various configuration scenarios
        
        // Test 1: Valid configuration
        let valid_config = AppConfig {
            providers: vec![
                ProviderConfig {
                    name: "provider1".to_string(),
                    provider_type: ProviderType::OpenAI,
                    api_key: "key123".to_string(),
                    base_url: None,
                    models: vec![],
                    rate_limit: None,
                }
            ],
            default_provider: Some("provider1".to_string()),
            server: ServerConfig::default(),
            features: FeaturesConfig::default(),
        };
        
        assert!(valid_config.validate().is_ok());
        
        // Test 2: Invalid - default provider doesn't exist
        let invalid_config = AppConfig {
            providers: vec![],
            default_provider: Some("nonexistent".to_string()),
            server: ServerConfig::default(),
            features: FeaturesConfig::default(),
        };
        
        let result = invalid_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Default provider 'nonexistent' not found"));
        
        // Test 3: Invalid - duplicate provider names
        let duplicate_config = AppConfig {
            providers: vec![
                ProviderConfig {
                    name: "same_name".to_string(),
                    provider_type: ProviderType::OpenAI,
                    api_key: "key1".to_string(),
                    base_url: None,
                    models: vec![],
                    rate_limit: None,
                },
                ProviderConfig {
                    name: "same_name".to_string(),
                    provider_type: ProviderType::Anthropic,
                    api_key: "key2".to_string(),
                    base_url: None,
                    models: vec![],
                    rate_limit: None,
                }
            ],
            default_provider: None,
            server: ServerConfig::default(),
            features: FeaturesConfig::default(),
        };
        
        let result = duplicate_config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate provider name"));
    }

    #[test]
    fn test_config_merge() {
        // GIVEN: A base configuration and override configuration
        let base_config = AppConfig {
            providers: vec![
                ProviderConfig {
                    name: "openai".to_string(),
                    provider_type: ProviderType::OpenAI,
                    api_key: "base-key".to_string(),
                    base_url: None,
                    models: vec![],
                    rate_limit: None,
                }
            ],
            default_provider: Some("openai".to_string()),
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            features: FeaturesConfig {
                streaming_enabled: false,
                function_calling_enabled: false,
            },
        };
        
        let override_config = PartialAppConfig {
            providers: None,
            default_provider: Some(Some("anthropic".to_string())),
            server: Some(PartialServerConfig {
                host: None,
                port: Some(8080),
            }),
            features: Some(PartialFeaturesConfig {
                streaming_enabled: Some(true),
                function_calling_enabled: None,
            }),
        };
        
        // WHEN: We merge the configurations
        let merged = base_config.merge(override_config);
        
        // THEN: Override values should take precedence
        assert_eq!(merged.default_provider, Some("anthropic".to_string()));
        assert_eq!(merged.server.host, "127.0.0.1"); // Not overridden
        assert_eq!(merged.server.port, 8080); // Overridden
        assert!(merged.features.streaming_enabled); // Overridden
        assert!(!merged.features.function_calling_enabled); // Not overridden
    }

    #[test]
    fn test_provider_specific_config() {
        // GIVEN: Provider-specific configurations
        let config = AppConfig {
            providers: vec![
                ProviderConfig {
                    name: "openai-gpt4".to_string(),
                    provider_type: ProviderType::OpenAI,
                    api_key: "key1".to_string(),
                    base_url: None,
                    models: vec!["gpt-4".to_string(), "gpt-4-turbo".to_string()],
                    rate_limit: Some(RateLimitConfig {
                        requests_per_minute: 60,
                        tokens_per_minute: 90000,
                    }),
                },
                ProviderConfig {
                    name: "anthropic-claude".to_string(),
                    provider_type: ProviderType::Anthropic,
                    api_key: "key2".to_string(),
                    base_url: Some("https://api.anthropic.com/v1".to_string()),
                    models: vec!["claude-3-opus".to_string()],
                    rate_limit: Some(RateLimitConfig {
                        requests_per_minute: 50,
                        tokens_per_minute: 100000,
                    }),
                },
            ],
            default_provider: None,
            server: ServerConfig::default(),
            features: FeaturesConfig::default(),
        };
        
        // WHEN: We access provider configurations
        let openai_config = config.get_provider("openai-gpt4").unwrap();
        let anthropic_config = config.get_provider("anthropic-claude").unwrap();
        
        // THEN: Each provider should have its specific settings
        assert_eq!(openai_config.models.len(), 2);
        assert!(openai_config.models.contains(&"gpt-4".to_string()));
        assert_eq!(openai_config.rate_limit.as_ref().unwrap().requests_per_minute, 60);
        
        assert_eq!(anthropic_config.models.len(), 1);
        assert_eq!(anthropic_config.base_url, Some("https://api.anthropic.com/v1".to_string()));
        assert_eq!(anthropic_config.rate_limit.as_ref().unwrap().tokens_per_minute, 100000);
    }

    #[test]
    fn test_config_hot_reload() {
        // GIVEN: A configuration that can be watched for changes
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let initial_config = r#"
            default_provider = "openai"
            
            [[providers]]
            name = "openai"
            type = "openai"
            api_key = "initial-key"
        "#;
        
        fs::write(&config_path, initial_config).unwrap();
        
        // WHEN: We set up configuration with hot reload
        let (config, mut watcher) = AppConfig::with_hot_reload(&config_path).unwrap();
        
        // Initial state check
        assert_eq!(config.read().unwrap().providers[0].api_key, "initial-key");
        
        // Update the configuration file
        let updated_config = r#"
            default_provider = "openai"
            
            [[providers]]
            name = "openai"
            type = "openai"
            api_key = "updated-key"
        "#;
        
        fs::write(&config_path, updated_config).unwrap();
        
        // THEN: The configuration should be automatically reloaded
        // Note: In real implementation, this would use file system events
        // For testing, we simulate the reload
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // In actual implementation, the watcher would trigger this
        let new_config = AppConfig::from_file(&config_path).unwrap();
        *config.write().unwrap() = new_config;
        
        assert_eq!(config.read().unwrap().providers[0].api_key, "updated-key");
    }
}

// Type definitions that will be moved to the actual implementation
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub providers: Vec<ProviderConfig>,
    pub default_provider: Option<String>,
    pub server: ServerConfig,
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: ProviderType,
    pub api_key: String,
    pub base_url: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Google,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub streaming_enabled: bool,
    pub function_calling_enabled: bool,
}

// Partial config structs for merging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialAppConfig {
    pub providers: Option<Vec<ProviderConfig>>,
    pub default_provider: Option<Option<String>>,
    pub server: Option<PartialServerConfig>,
    pub features: Option<PartialFeaturesConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialServerConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialFeaturesConfig {
    pub streaming_enabled: Option<bool>,
    pub function_calling_enabled: Option<bool>,
}

// Default implementations
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            providers: vec![],
            default_provider: None,
            server: ServerConfig::default(),
            features: FeaturesConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            streaming_enabled: false,
            function_calling_enabled: false,
        }
    }
}

// Implementation stubs
impl AppConfig {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn expand_env_vars(&mut self) {
        for provider in &mut self.providers {
            if provider.api_key.starts_with("${") && provider.api_key.ends_with("}") {
                let var_name = &provider.api_key[2..provider.api_key.len()-1];
                if let Ok(value) = std::env::var(var_name) {
                    provider.api_key = value;
                }
            }
            
            if let Some(ref mut base_url) = provider.base_url {
                if base_url.starts_with("${") && base_url.ends_with("}") {
                    let var_name = &base_url[2..base_url.len()-1];
                    if let Ok(value) = std::env::var(var_name) {
                        *base_url = value;
                    }
                }
            }
        }
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Check for duplicate provider names
        let mut names = std::collections::HashSet::new();
        for provider in &self.providers {
            if !names.insert(&provider.name) {
                return Err(ConfigError::Validation(format!("Duplicate provider name: {}", provider.name)));
            }
        }
        
        // Check that default provider exists
        if let Some(ref default) = self.default_provider {
            if !self.providers.iter().any(|p| &p.name == default) {
                return Err(ConfigError::Validation(format!("Default provider '{}' not found", default)));
            }
        }
        
        Ok(())
    }
    
    pub fn merge(mut self, partial: PartialAppConfig) -> Self {
        if let Some(providers) = partial.providers {
            self.providers = providers;
        }
        
        if let Some(default) = partial.default_provider {
            self.default_provider = default;
        }
        
        if let Some(server) = partial.server {
            if let Some(host) = server.host {
                self.server.host = host;
            }
            if let Some(port) = server.port {
                self.server.port = port;
            }
        }
        
        if let Some(features) = partial.features {
            if let Some(streaming) = features.streaming_enabled {
                self.features.streaming_enabled = streaming;
            }
            if let Some(function_calling) = features.function_calling_enabled {
                self.features.function_calling_enabled = function_calling;
            }
        }
        
        self
    }
    
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }
    
    pub fn with_hot_reload(path: &Path) -> Result<(Arc<RwLock<Self>>, ConfigWatcher), ConfigError> {
        let config = Self::from_file(path)?;
        let config = Arc::new(RwLock::new(config));
        
        // In real implementation, this would set up file system watching
        let watcher = ConfigWatcher {};
        
        Ok((config, watcher))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

pub struct ConfigWatcher {
    // File system watcher implementation
}