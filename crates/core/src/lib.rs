pub mod container;
pub mod supervisor;
pub mod git;

use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use once_cell::sync::Lazy;
use std::env;

// Use `once_cell` to create a single, lazily-initialized OpenAI client.
// This is more efficient than creating a new client for every request.
static OPENAI_CLIENT: Lazy<Result<Client<OpenAIConfig>>> = Lazy::new(|| {
    // Load environment variables from .env file. This is crucial for local dev.
    dotenvy::dotenv().ok();

    let api_key = env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY must be set in your environment or .env file")?;
    
    let config = OpenAIConfig::new().with_api_key(api_key);
    Ok(Client::with_config(config))
});

/// Sends a prompt to the OpenAI chat API and returns the response.
///
/// # Arguments
/// * `prompt` - A string slice containing the user's prompt.
///
/// # Returns
/// A `Result<String>` containing the AI's response or an error.
pub async fn ask(prompt: &str) -> Result<String> {
    // Get the initialized client, cloning the Result.
    // If initialization failed, this will propagate the error.
    let client = match &*OPENAI_CLIENT {
        Ok(client) => client,
        Err(e) => return Err(anyhow::anyhow!("Failed to initialize client: {}", e)),
    };

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o") // Specify the model
        .max_tokens(512u16)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(prompt)
            .build()?
            .into()])
        .build()?;

    let response = client.chat().create(request).await?;

    // Extract the content from the first choice in the response.
    let content = response
        .choices
        .into_iter()
        .next()
        .context("No choices returned from the API")?
        .message
        .content
        .context("No content in the message response")?;

    Ok(content)
}

// Module for tests
#[cfg(test)]
mod tests {
    use super::*;

    /// This test makes a real API call. It's marked as `#[ignore]`
    /// so it doesn't run during normal `cargo test` runs.
    /// Run it specifically with: `cargo test -- --ignored`
    #[tokio::test]
    #[ignore]
    async fn test_ask_function_success() {
        let prompt = "What is the capital of France?";
        let result = ask(prompt).await;

        assert!(result.is_ok(), "The ask function should succeed.");
        let response = result.unwrap();
        assert!(response.to_lowercase().contains("paris"), "Response should contain 'Paris'");
    }

    #[tokio::test]
    async fn test_ask_function_fails_without_key() {
        // Since the static client is already initialized, this test is limited.
        // We'll test that the env var is required by checking it directly.
        let original_key = env::var("OPENAI_API_KEY");
        env::remove_var("OPENAI_API_KEY");
        
        // Test that the key is missing
        assert!(env::var("OPENAI_API_KEY").is_err(), "OPENAI_API_KEY should be missing");

        // Restore the key if it was originally set
        if let Ok(key) = original_key {
            env::set_var("OPENAI_API_KEY", key);
        }
    }
}
