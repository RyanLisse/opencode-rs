use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests;

/// OpenAI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub default_model: String,
    pub api_base: String,
    pub max_retries: u32,
    pub timeout_seconds: u32,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            default_model: "gpt-4".to_string(),
            api_base: "https://api.openai.com/v1".to_string(),
            max_retries: 3,
            timeout_seconds: 30,
        }
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub openai: OpenAIConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai: OpenAIConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file and environment variables
    /// Environment variables take precedence over file values
    pub fn load<P: AsRef<Path>>(config_path: Option<P>) -> Result<Self> {
        let mut config = if let Some(path) = config_path {
            Self::from_file(path)?
        } else {
            Self::default()
        };

        // Override with environment variables
        let env_config = Self::from_env()?;
        config.merge_env(env_config);

        Ok(config)
    }

    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // OpenAI configuration
        if let Ok(model) = env::var("OPENAI_MODEL") {
            config.openai.default_model = model;
        }

        if let Ok(api_base) = env::var("OPENAI_API_BASE") {
            config.openai.api_base = api_base;
        }

        if let Ok(max_retries) = env::var("OPENAI_MAX_RETRIES") {
            config.openai.max_retries = max_retries
                .parse()
                .map_err(|e| Error::Config(format!("Invalid OPENAI_MAX_RETRIES: {}", e)))?;
        }

        if let Ok(timeout) = env::var("OPENAI_TIMEOUT") {
            config.openai.timeout_seconds = timeout
                .parse()
                .map_err(|e| Error::Config(format!("Invalid OPENAI_TIMEOUT: {}", e)))?;
        }

        Ok(config)
    }

    /// Merge environment configuration into this config
    /// Environment values take precedence
    fn merge_env(&mut self, env_config: Config) {
        // Only update values that were actually set in environment
        if env::var("OPENAI_MODEL").is_ok() {
            self.openai.default_model = env_config.openai.default_model;
        }
        if env::var("OPENAI_API_BASE").is_ok() {
            self.openai.api_base = env_config.openai.api_base;
        }
        if env::var("OPENAI_MAX_RETRIES").is_ok() {
            self.openai.max_retries = env_config.openai.max_retries;
        }
        if env::var("OPENAI_TIMEOUT").is_ok() {
            self.openai.timeout_seconds = env_config.openai.timeout_seconds;
        }
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)?;
        Ok(())
    }
}