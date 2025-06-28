Of course. Here is the detailed guide for the second vertical slice, building upon the foundation laid in Slice 1.

***

### **Vertical Slice 2: CLI Chat MVP**

With the core logic in place, this slice focuses on creating a user-facing command-line application. You will build an `opencode` binary that provides a simple, interactive chat REPL (Read-Eval-Print Loop), making the `ask` function truly usable.

---

### **Part 1: Prerequisites & Setup**

First, prepare your workspace for this new feature. It's assumed that Slice 1 has been "merged" into the `main` branch.

**1. Update Your Local `main` Branch:**
Navigate back to the directory of your `main` branch worktree (the original `opencode-rs` directory) and ensure it's up to date.
```bash
# Navigate to the main worktree directory
cd ../opencode-rs

# (If Slice 1 PR was actually merged, pull the changes)
# git switch main
# git pull origin main

# For this exercise, we'll merge our slice-1 branch into main locally
git switch main
git merge --no-ff slice-1-core-skeleton

# Now, remove the old worktree for the completed slice
git worktree remove ../opencode-rs-slice-1
git branch -d slice-1-core-skeleton
```

**2. Create a New `git worktree` for Slice 2:**
Just like before, we'll create a dedicated branch and directory for this slice's work.

```bash
# From the `opencode-rs` directory:

# 1. Create the new branch and worktree directory
git worktree add -B slice-2-cli-mvp ../opencode-rs-slice-2

# 2. Navigate into your new work directory.
cd ../opencode-rs-slice-2

# All work for Slice 2 will be done in this directory.
```

---

### **Part 2: Implementing Slice 2**

Now, let's build the CLI application.

#### **What You’re Building**
An `opencode` binary crate. It will have a `chat` command that can either take a single prompt as an argument or, if no prompt is given, launch an interactive REPL that continuously chats with the AI.

#### **Step-by-Step Instructions**

**Step 1: Create the `cli` Crate and Add Dependencies**

1.  From your worktree directory (`opencode-rs-slice-2`), create the new binary crate.
    ```bash
    cargo new crates/cli --bin
    ```
2.  Open `crates/cli/Cargo.toml` and add the necessary dependencies.
    ```toml
    [package]
    name = "opencode" # The binary will be named 'opencode'
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    # Get dependencies from the workspace root
    tokio = { workspace = true }
    anyhow = { workspace = true }

    # Crate-specific dependencies
    opencode_core = { path = "../core" } # Add the core crate
    clap = { version = "4.5", features = ["derive"] }
    reedline = "0.31"
    ```

**Step 2: Add the `cli` Crate to the Workspace**

1.  Open the main `Cargo.toml` file in your worktree root (`opencode-rs-slice-2/Cargo.toml`).
2.  Uncomment (or add) `crates/cli` to the `members` array.

    ```toml
    [workspace]
    resolver = "2"
    members = [
        "crates/core",
        "crates/cli", # Add this line
    ]
    # ... rest of the file
    ```

**Step 3: Implement the CLI Logic**

Replace the contents of `crates/cli/src/main.rs` with the following code. This code uses `clap` to define the command structure and `reedline` for the interactive loop.

**`crates/cli/src/main.rs`**
```rust
use anyhow::Result;
use clap::{Parser, Subcommand};
use opencode_core::ask;
use reedline::{DefaultPrompt, Reedline, Signal};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Chat with the AI model
    Chat {
        /// The prompt to send to the AI. If omitted, enters interactive mode.
        prompt: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Chat { prompt } => {
            // Check if a prompt was passed directly
            if let Some(p) = prompt {
                // Single-shot mode
                println!("You: {}", p);
                let response = ask(p).await?;
                println!("\nAI: {}", response);
            } else {
                // Interactive mode
                enter_interactive_mode().await?;
            }
        }
    }

    Ok(())
}

/// Enters an interactive REPL loop to chat with the AI.
async fn enter_interactive_mode() -> Result<()> {
    println!("Entering interactive chat mode. Press Ctrl+C or Ctrl+D to exit.");

    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::new("You: ".into(), "".into());

    loop {
        // Read a line from the user
        let sig = line_editor.read_line(&prompt)?;

        match sig {
            Signal::Success(line) => {
                if line.trim().is_empty() {
                    continue; // Skip empty lines
                }
                // Send the line to the AI
                let response = ask(&line).await?;
                println!("\nAI: {}\n", response);
            }
            Signal::CtrlC | Signal::CtrlD => {
                // User wants to exit
                println!("Exiting chat.");
                break;
            }
        }
    }
    Ok(())
}
```

**Step 4: Test Your CLI Manually**

The `Makefile` you created in Slice 1 still works perfectly, as its commands operate on the entire workspace.

1.  First, ensure everything compiles.
    ```bash
    make build
    # Or: cargo build --workspace
    ```
2.  Test the **single-shot mode**:
    ```bash
    cargo run -- chat "What is Rust's mascot?"
    ```
    You should see your prompt, and then a response from the AI about "Ferris the crab".

3.  Test the **interactive mode**:
    ```bash
    cargo run -- chat
    ```
    You will see the "Entering interactive chat mode" message and a `You:` prompt. You can now have a conversation.
    *   Type a question and press Enter.
    *   Try entering multiline input (it might not be enabled by default in `reedline` but basic entry works).
    *   Press `Ctrl+C` or `Ctrl+D` to exit the loop gracefully.

---

### **Part 3: Final Review & Merge Preparation**

You've built a working CLI! Let's get it ready for review.

#### **Ready to Merge Checklist**
*   [x] **Works on macOS & Linux:** The dependencies used are cross-platform.
*   [x] **`cargo run --bin opencode chat "hi"` returns answer:** You verified this in the manual testing step.
*   [x] **Added to workspace Cargo.toml:** You completed this in Step 2.
*   [ ] **Commit your work:** Stage and commit all your changes to the `slice-2-cli-mvp` branch.
    ```bash
    git add .
    git commit -m "feat(cli): Implement Slice 2 - CLI chat MVP with interactive mode"
    ```
*   [ ] **Push and open a Pull Request:**
    ```bash
    git push --set-upstream origin slice-2-cli-mvp
    ```
    Go to your Git provider and open a Pull Request from `slice-2-cli-mvp` to `main`.

#### **Question for Senior Dev**
Include this question in your PR to stimulate discussion on the next steps:
> *   Should we add colourised output (e.g., for prompts and AI responses) now, or is that a polish item for later?

You are now ready to move on to Slice 3, where you'll add more powerful features like slash commands.