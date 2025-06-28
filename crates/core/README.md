# OpenCode Core

This crate contains the core business logic for the OpenCode-RS application.

## Usage

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

## Environment Setup

This crate requires an OpenAI API key. Set it in your environment:

```bash
export OPENAI_API_KEY="sk-your-key-here"
```

Or create a `.env` file in the project root:

```
OPENAI_API_KEY="sk-your-key-here"
```

## Testing

Run the unit tests (excludes API tests):
```bash
cargo test
```

Run all tests including API integration tests:
```bash
cargo test -- --ignored
```