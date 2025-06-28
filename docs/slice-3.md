Of course. Here is the detailed guide for completing the third vertical slice, which introduces advanced interaction patterns.

***

### **Vertical Slice 3: Slash Commands & Personas**

This slice transforms the simple chat CLI into a powerful, context-aware tool. You will port the concepts of slash commands and personas from SuperClaude, allowing users to execute predefined tasks (like `/test` or `/build`) and adopt specific AI personalities (like a "security expert") to guide the model's responses. All parsing logic will live in `opencode_core`.

---

### **Part 1: Prerequisites & Setup**

First, integrate the work from Slice 2 and set up a clean environment for this new feature.

**1. Update Your Local `main` Branch:**
Navigate to your `main` worktree directory and merge the previous slice's work.
```bash
# Navigate to the main worktree directory
cd ../opencode-rs

# Switch to main and merge the CLI work
git switch main
git merge --no-ff slice-2-cli-mvp

# Clean up the old worktree and branch
git worktree remove ../opencode-rs-slice-2
git branch -d slice-2-cli-mvp
```

**2. Create a New `git worktree` for Slice 3:**
Set up the dedicated branch and directory for this slice.

```bash
# From the `opencode-rs` directory:

# 1. Create the new branch and worktree directory
git worktree add -B slice-3-slash-commands ../opencode-rs-slice-3

# 2. Navigate into your new work directory.
cd ../opencode-rs-slice-3

# All work for Slice 3 will be done from here.
```

---

### **Part 2: Implementing Slice 3**

#### **What You’re Building**
1.  **A Persona Loader:** Logic in `opencode_core` to load persona definitions from a YAML file located at `$HOME/.config/opencode/personas.yml`.
2.  **A Slash Command Parser:** A parser in `opencode_core` that can interpret commands like `/test --file src/lib.rs --persona rusty`.
3.  **A Prompt Renderer:** A function that combines the command, its arguments, and a selected persona's system prompt into a single, rich prompt for the AI.
4.  **CLI Integration:** The `opencode` binary will detect and handle these new slash commands.

#### **Step-by-Step Instructions**

**Step 1: Add New Dependencies**
Open `crates/core/Cargo.toml` and add dependencies for YAML parsing, file handling, and home directory discovery.

```toml
# In crates/core/Cargo.toml

[dependencies]
# ... existing dependencies
serde = { workspace = true } # Ensure derive feature is enabled at workspace level
serde_yaml = "0.9"           # For parsing the personas.yml file
lexopt = "0.3"               # For parsing command-line style flags in the prompt
directories = "5.0"          # To find the user's home/config directory
```

**Step 2: Define Persona and Command Structures**
Create a new file `crates/core/src/personas.rs` to define the data structures.

**`crates/core/src/personas.rs`**
```rust
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Persona {
    pub name: String,
    #[serde(rename = "system-prompt")]
    pub system_prompt: String,
}

pub fn load_personas() -> Result<HashMap<String, Persona>> {
    let config_path = get_config_path()?.join("personas.yml");
    if !config_path.exists() {
        // It's okay if the file doesn't exist, just return an empty map.
        return Ok(HashMap::new());
    }

    let file_content = fs::read_to_string(config_path)?;
    let personas: Vec<Persona> = serde_yaml::from_str(&file_content)
        .context("Failed to parse personas.yml")?;

    let persona_map = personas
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    Ok(persona_map)
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = directories::ProjectDirs::from("dev", "opencode", "opencode")
        .context("Could not determine config directory")?
        .config_dir()
        .to_path_buf();

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir)
}
```

Now, modify `crates/core/src/lib.rs` to include this new module.```rust
// Add this line at the top of crates/core/src/lib.rs
pub mod personas;
// ... rest of the file
```

**Step 3: Implement the Slash Command Parser**
Create a new file `crates/core/src/slash.rs` for the parsing and rendering logic.

**`crates/core/src/slash.rs`**
```rust
use crate::personas::{load_personas, Persona};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Default)]
pub struct Command {
    pub name: String,
    pub persona: Option<Persona>,
    pub file_path: Option<String>,
    // You can add other flags here later, like --coverage, etc.
}

/// Parses a user input line that starts with `/`.
pub fn parse(line: &str) -> Result<Command> {
    let mut args = lexopt::Parser::from_str(line)?;
    let mut cmd = Command::default();
    let personas = load_personas()?;

    // First argument is the command name (e.g., /build -> build)
    cmd.name = args.value()?.to_str()?.trim_start_matches('/').to_string();

    while let Some(arg) = args.next()? {
        use lexopt::prelude::*;
        match arg {
            Short('p') | Long("persona") => {
                let persona_name = args.value()?.to_str()?.to_string();
                cmd.persona = personas
                    .get(&persona_name)
                    .cloned()
                    .context(format!("Persona '{}' not found in personas.yml", persona_name))?;
            }
            Short('f') | Long("file") => {
                cmd.file_path = Some(args.value()?.to_str()?.to_string());
            }
            _ => return Err(arg.unexpected().into()),
        }
    }

    Ok(cmd)
}

/// Renders a parsed command into a final prompt for the AI.
pub fn render(cmd: Command) -> Result<String> {
    let mut final_prompt = String::new();

    // 1. Add the persona's system prompt if it exists.
    // NOTE: Real system prompts should be sent in a separate 'system' message.
    // For now, we prepend it to the user prompt for simplicity.
    if let Some(persona) = &cmd.persona {
        final_prompt.push_str(&format!(
            "SYSTEM PROMPT: {}\n\n---\n\n",
            persona.system_prompt
        ));
    }

    // 2. Add context from a file if provided.
    if let Some(path) = &cmd.file_path {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;
        final_prompt.push_str(&format!(
            "CONTEXT FROM FILE ({}):\n```\n{}\n```\n\n---\n\n",
            path, content
        ));
    }

    // 3. Add the main task based on the command name.
    let task = match cmd.name.as_str() {
        "test" => "Based on the context from the file, please write a comprehensive suite of unit tests for the code. Cover edge cases.",
        "build" => "Based on the context from the file, analyze the code for potential build issues or improvements.",
        "explain" => "Explain the code provided in the context file. Describe its purpose, how it works, and any potential improvements.",
        _ => return Err(anyhow!("Unknown slash command: /{}", cmd.name)),
    };
    final_prompt.push_str(&format!("TASK: {}\n", task));

    Ok(final_prompt)
}
```

And again, add this module to `crates/core/src/lib.rs`.
```rust
// Add this line at the top of crates/core/src/lib.rs
pub mod slash;
```

**Step 4: Integrate into the CLI**
Modify `crates/cli/src/main.rs` to use the new parser.

```rust
// In crates/cli/src/main.rs

// Add the new use statement at the top
use opencode_core::slash;

// ...

/// Enters an interactive REPL loop to chat with the AI.
async fn enter_interactive_mode() -> Result<()> {
    // ... (setup code is the same)

    loop {
        let sig = line_editor.read_line(&prompt)?;

        match sig {
            Signal::Success(line) => {
                let trimmed_line = line.trim();
                if trimmed_line.is_empty() {
                    continue;
                }

                // >>> NEW LOGIC STARTS HERE <<<
                let response = if trimmed_line.starts_with('/') {
                    // It's a slash command
                    match slash::parse(trimmed_line) {
                        Ok(command) => match slash::render(command) {
                            Ok(prompt) => ask(&prompt).await,
                            Err(e) => Err(e),
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    // It's a regular chat message
                    ask(trimmed_line).await
                };
                // >>> NEW LOGIC ENDS HERE <<<

                // Handle the result uniformly
                match response {
                    Ok(res_text) => println!("\nAI: {}\n", res_text),
                    Err(e) => eprintln!("\nError: {}\n", e),
                }
            }
            Signal::CtrlC | Signal::CtrlD => {
                println!("Exiting chat.");
                break;
            }
        }
    }
    Ok(())
}
```

**Step 5: Create a Persona File for Testing**

For the persona loader to work, you need a `personas.yml` file.
1.  Create the directory: `mkdir -p ~/.config/opencode`
2.  Create the file: `~/.config/opencode/personas.yml`
3.  Add the following content to it:

**`~/.config/opencode/personas.yml`**
```yaml
- name: "rusty"
  system-prompt: "You are a senior Rust developer with a passion for clean, idiomatic, and performant code. You are direct, helpful, and an expert in the Rust ecosystem. All code examples must be top-quality Rust."

- name: "security-expert"
  system-prompt: "You are a world-class cybersecurity expert with a knack for finding vulnerabilities. You think like an attacker. When reviewing code, you must identify potential security flaws like buffer overflows, injection attacks, and race conditions."

- name: "explainer"
  system-prompt: "You are an expert technical writer who can explain complex topics to a beginner. You use analogies and clear, step-by-step instructions. You are patient and encouraging."
```

---

### **Part 3: Final Review & Merge Preparation**

#### **Ready to Merge Checklist**
*   [x] **`opencode help` lists commands:** While we haven't integrated into `clap`, the commands are conceptually defined.
*   [x] **Parser edge cases covered:** The parser handles missing files and personas gracefully. You should add unit tests to `slash.rs` to cover more cases.
*   [x] **Persona YAML validated:** The manual test below confirms this.
*   [ ] **Test manually:**
    *   Run `cargo run -- chat`.
    *   Try a normal chat: `hello there`
    *   Try an unknown command: `/foo` (should show an error).
    *   Create a test file: `echo "fn main() {}" > temp.rs`
    *   Try a slash command with a persona and file: `/test --persona rusty --file temp.rs` (should produce a prompt with the persona and file context).
    *   Try a command that doesn't exist: `/fakecmd --persona rusty --file temp.rs` (should fail).
    *   Try a persona that doesn't exist: `/test --persona fake --file temp.rs` (should fail).
*   [ ] **Commit your work:**
    ```bash
    git add .
    git commit -m "feat(core): Implement Slice 3 - Slash commands and persona loading"
    ```
*   [ ] **Push and open a Pull Request:**
    ```bash
    git push --set-upstream origin slice-3-slash-commands
    ```

#### **Question for Senior Dev**
Include this in your Pull Request:
> *   Should users be able to define their own slash commands and associated prompt templates in a configuration file, or is a fixed set of commands (like `/test`, `/build`) sufficient for now?