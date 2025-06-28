Of course. Here is the detailed guide for completing the first vertical slice, including the initial project setup and a `Makefile` for automation. This guide is designed for a junior developer to follow step-by-step.

***

### **Vertical Slice 1: Core Skeleton & Project Setup**

This first slice establishes the foundation of the entire project. We will create the workspace, build the core logic crate, and implement the most fundamental feature: the ability to ask a question to an AI model and get a response.

---

### **Part 1: Prerequisites & Initial Setup**

Before writing any Rust code, we need to set up the project structure and development environment.

**1. Install Prerequisites:**
*   **Rust Toolchain:** Install via [rustup](https://rustup.rs/).
*   **Git:** Must be installed on your system.
*   **Docker Desktop** (or Colima on macOS): While not used in this slice, it's a core dependency for the project. Install it now to avoid issues later.
*   **OpenAI API Key:** Get a key from the [OpenAI Platform](https://platform.openai.com/).

**2. Create the Project Workspace:**
This project will contain multiple crates (`core`, `cli`, `gui`). A Cargo workspace is the right tool to manage this.

```bash
# 1. Create the root directory for the project
mkdir opencode-rs
cd opencode-rs

# 2. Create the main workspace Cargo.toml file
# This file declares the members of our workspace.
cat << EOF > Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    # "crates/cli", # We'll uncomment this in a later slice
]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-openai = "0.19"
dotenvy = "0.15"
# Add other shared dependencies here later
EOF

# 3. Create the directory to hold our crates
mkdir crates

# 4. Initialize a Git repository
git init
cat << EOF > .gitignore
# Ignore IDE and OS files
.idea/
.vscode/
.DS_Store

# Ignore build artifacts
/target/
Cargo.lock

# Ignore local environment files
.env
EOF

git add .
git commit -m "Initial commit: Set up workspace structure"
git branch -M main
```

**3. Set Up Your `git worktree` for This Slice:**
We will follow the PRD's workflow. All work for this slice will be done on a separate branch and in a separate directory to keep the `main` branch clean.

```bash
# From the `opencode-rs` directory:

# 1. Create a new branch `slice-1-core-skeleton` and a new directory 
#    `../opencode-rs-slice-1` where the branch will be checked out.
git worktree add -B slice-1-core-skeleton ../opencode-rs-slice-1

# 2. Navigate into your new work directory. This is where you'll work.
cd ../opencode-rs-slice-1

# You are now in a separate directory, but it's the same Git repo on a different branch.
# All commands below should be run from here (`opencode-rs-slice-1`).
```

---

### **Part 2: Makefile for Automation**

To streamline development, we'll use a `Makefile`. Create this file in the root of your worktree directory (`opencode-rs-slice-1/Makefile`).

**`Makefile`**```makefile
# Makefile for the OpenCode-RS Project

# Use .PHONY to ensure these targets run even if files with the same name exist.
.PHONY: all build check test lint clean help

# Default target runs when you just type 'make'
all: build

## --------------------------------------
## Development Commands
## --------------------------------------

# Build all crates in the workspace in debug mode
build:
	@echo "Building workspace..."
	@cargo build --workspace

# Check all crates for errors without building executables (faster)
check:
	@echo "Checking workspace..."
	@cargo check --workspace

# Run all tests in the workspace
test:
	@echo "Running tests..."
	@cargo test --workspace -- --nocapture

# Run the linter (clippy) and fail on any warnings
lint:
	@echo "Linting workspace..."
	@cargo clippy --workspace -- -D warnings

## --------------------------------------
## Housekeeping
## --------------------------------------

# Clean up build artifacts
clean:
	@echo "Cleaning workspace..."
	@cargo clean

# Self-documenting help command
help:
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

```

You can now run commands like `make test` and `make lint` from your terminal.

---

### **Part 3: Implementing Slice 1**

Now we'll build the actual feature as defined in the PRD.

#### **What You’re Building**
A minimal `opencode_core` crate that exposes an `async fn ask(prompt: &str)` function. This function will read an `OPENAI_API_KEY` from a `.env` file and use it to send the prompt to the OpenAI API.

#### **Step-by-Step Instructions**

**Step 1: Create the `core` Crate and Add Dependencies**
1.  From your worktree directory (`opencode-rs-slice-1`), create the new library crate.
    ```bash
    cargo new --lib crates/core
    ```
2.  Open `crates/core/Cargo.toml` and add the necessary dependencies. It should look like this:
    ```toml
    [package]
    name = "opencode_core"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    # Get dependencies from the workspace root
    tokio = { workspace = true }
    anyhow = { workspace = true }
    async-openai = { workspace = true }
    dotenvy = { workspace = true }

    # Crate-specific dependencies
    once_cell = "1.19" # For creating a global singleton client
    ```

**Step 2: Create the `.env` File**
Create a new file named `.env` in the root of your worktree (`opencode-rs-slice-1/.env`). **Remember, this file is in your `.gitignore` and should never be committed.**

```
# .env
OPENAI_API_KEY="sk-YourSecretKeyHere"
```

**Step 3: Implement the Core Logic**
Replace the contents of `crates/core/src/lib.rs` with the following code:

**`crates/core/src/lib.rs`**
```rust
use anyhow::{Context, Result};
use async_openai::{
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use once_cell::sync::Lazy;
use std::env;

// Use `once_cell` to create a single, lazily-initialized OpenAI client.
// This is more efficient than creating a new client for every request.
static OPENAI_CLIENT: Lazy<Result<Client>> = Lazy::new(|| {
    // Load environment variables from .env file. This is crucial for local dev.
    dotenvy::dotenv().ok();

    let api_key = env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY must be set in your environment or .env file")?;
    
    Ok(Client::new().with_api_key(api_key))
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
        // Temporarily unset the env var to simulate a missing key
        let original_key = env::var("OPENAI_API_KEY");
        env::remove_var("OPENAI_API_KEY");
        
        // Re-initialize the client by creating a temporary one
        let temp_client = Lazy::new(|| {
            let api_key = env::var("OPENAI_API_KEY")
                .context("OPENAI_API_KEY must be set");
            api_key.map(|key| Client::new().with_api_key(key))
        });
        
        // Force initialization and check that it's an error
        assert!(temp_client.is_err(), "Client initialization should fail without an API key.");

        // Restore the key if it was originally set
        if let Ok(key) = original_key {
            env::set_var("OPENAI_API_KEY", key);
        }
    }
}
```

**Step 4: Run the Tests**
Use the `Makefile` to verify your implementation.

```bash
# Run the unit tests (this will skip the ignored test)
make test

# To run the ignored test that makes a real API call:
cargo test --workspace -- --ignored
```
You should see both tests pass.

---

### **Part 4: Final Review & Merge Preparation**

You have completed the first slice! Before creating a Pull Request, follow this checklist.

#### **Ready to Merge Checklist**
*   [x] **`make test` passes:** All unit tests are green.
*   [x] **`make lint` passes:** The code is free of warnings and follows Rust conventions.
*   [ ] **README is updated:** Create a `crates/core/README.md` and add a small usage snippet.
    ```markdown
    # OpenCode Core

    This crate contains the core business logic for the OpenCode-RS application.

    ### Usage

    ```rust
    use opencode_core::ask;

    #[tokio::main]
    async fn main() {
        match ask("Hello, AI!").await {
            Ok(response) => println!("AI says: {}", response),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    ```
    ```
*   [ ] **Commit your work:** Stage and commit all your changes to the `slice-1-core-skeleton` branch.
    ```bash
    git add .
    git commit -m "feat(core): Implement Slice 1 - Core skeleton with ask function"
    ```*   [ ] **Push and open a Pull Request:**
    ```bash
    git push --set-upstream origin slice-1-core-skeleton
    ```
    Now go to your Git hosting provider (GitHub, GitLab) and open a Pull Request from the `slice-1-core-skeleton` branch to `main`.

#### **Questions for Senior Dev**
When you create your Pull Request, include these questions from the PRD in the description to show forethought:
> *   Is a gRPC abstraction for providers needed now, or can we defer that?
> *   Should we hide the model name (`gpt-4o`) behind a configuration file, or is hardcoding it acceptable for now?

You are now ready for the next slice