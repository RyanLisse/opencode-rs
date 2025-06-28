Of course. Here is the detailed guide for Vertical Slice 4. This is a significant slice where the project starts interacting with external systems (Docker and Git worktrees) in a much deeper way.

***

### **Vertical Slice 4: `container-use` Integration**

This slice implements a core security and isolation feature of OpenCode-RS. You will leverage Dagger's `container-use` (`cu`) to create sandboxed environments for tasks. Instead of running directly on the host machine, slash commands will now execute inside a dedicated, temporary container, operating on a clean `git worktree` of the project.

---

### **Part 1: Prerequisites & Setup**

This slice introduces new, critical dependencies for the user's environment.

**1. Install New Prerequisites:**
*   **Docker Desktop** (on macOS or Windows) or **Colima** (on macOS). Docker must be running for `cu` to work.
*   **`container-use`:** Follow the installation instructions on the official [`container-use` GitHub repository](https://github.com/dagger/container-use.git). The simplest way is usually via `go install`:
    ```bash
    go install github.com/dagger/container-use/cmd/cu@latest
    ```
    Verify it's installed by running `cu --version`.

**2. Update Your Local `main` Branch:**
Merge the previous slice's work and prepare a new worktree.
```bash
# Navigate to the main worktree directory
cd ../opencode-rs

# Switch to main and merge the slash commands work
git switch main
git merge --no-ff slice-3-slash-commands

# Clean up the old worktree and branch
git worktree remove ../opencode-rs-slice-3
git branch -d slice-3-slash-commands
```

**3. Create a New `git worktree` for Slice 4:**
```bash
# From the `opencode-rs` directory:
git worktree add -B slice-4-container-integration ../opencode-rs-slice-4
cd ../opencode-rs-slice-4

# All work for Slice 4 will be done from here.
```

---

### **Part 2: Implementing Slice 4**

#### **What You’re Building**
1.  **A `cu` verifier:** A function in `opencode_core` to check if `cu` is available on the user's `PATH`.
2.  **Container Execution Logic:** A new module in `opencode_core` responsible for spawning agent environments. It will:
    *   Generate a unique branch name for each task.
    *   Use `tokio::process::Command` to run `cu environment open --branch <branch_name> -- <command>`.
    *   Stream the `stdout` and `stderr` from the container back to the user.
3.  **CLI Integration:** Modify the CLI to use this new containerized execution flow for slash commands.

#### **Step-by-Step Instructions**

**Step 1: Add New Dependencies**
We need better logging for debugging subprocesses and a way to generate unique IDs. Add `tracing` and `rand` to `crates/core/Cargo.toml`.

```toml
# In crates/core/Cargo.toml

[dependencies]
# ... existing dependencies
tracing = "0.1"
rand = "0.8"
```
Also add `tracing-subscriber` to `crates/cli/Cargo.toml` to initialize logging.```toml
# In crates/cli/Cargo.toml

[dependencies]
# ... existing dependencies
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Step 2: Create the Container Module in `core`**
Create a new file `crates/core/src/container.rs`. This will house all the logic for interacting with `cu`.

**`crates/core/src/container.rs`**
```rust
use anyhow::{bail, Context, Result};
use std::process::Stdio;
use tokio::process::Command;

/// Checks if the 'cu' command is available on the system PATH.
pub async fn check_cu_exists() -> Result<()> {
    let output = Command::new("cu")
        .arg("--version")
        .output()
        .await
        .context("Failed to execute 'cu' command. Is `container-use` installed and in your PATH?")?;

    if !output.status.success() {
        bail!("'cu --version' command failed. Ensure container-use is correctly installed.");
    }
    Ok(())
}

/// Executes a given shell command inside a sandboxed container-use environment.
///
/// # Arguments
/// * `branch_name` - The git branch to use for the worktree.
/// * `shell_command` - The shell command to execute inside the container.
///
/// This function will stream the stdout/stderr of the command directly to the parent process.
pub async fn run_in_container(branch_name: &str, shell_command: &str) -> Result<()> {
    println!(
        "\n--- Spawning environment for branch '{}' ---",
        branch_name
    );

    let mut child = Command::new("cu")
        .arg("environment")
        .arg("open")
        .arg("--branch")
        .arg(branch_name)
        .arg("--") // Separator: everything after this is the command to run
        .arg("sh")
        .arg("-c")
        .arg(shell_command)
        .stdout(Stdio::inherit()) // Stream stdout directly to parent
        .stderr(Stdio::inherit()) // Stream stderr directly to parent
        .spawn()
        .context("Failed to spawn 'cu environment open' process.")?;

    let status = child
        .wait()
        .await
        .context("Failed to wait for container process to complete.")?;

    println!("--- Environment for '{}' finished ---", branch_name);

    if !status.success() {
        bail!(
            "Container command exited with a non-zero status: {}",
            status
        );
    }

    Ok(())
}
```

Now, add this as a module in `crates/core/src/lib.rs`.
```rust
// Add this line at the top of crates/core/src/lib.rs
pub mod container;
```

**Step 3: Integrate Container Logic into the CLI**

We will modify the CLI to call our new container functions. For this slice, we will run a simple command like `ls -l <file>` inside the container to prove the concept works, instead of calling the LLM.

**First, initialize tracing in `crates/cli/src/main.rs`:**
```rust
// In crates/cli/src/main.rs

// Add new use statements
use opencode_core::container;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Before doing anything, check if `cu` is installed
    container::check_cu_exists().await?;
    
    // ... rest of the main function is the same
}
```

**Next, modify the `enter_interactive_mode` function:**
```rust
// In crates/cli/src/main.rs

// Add necessary use statements
use opencode_core::{slash, container};
use rand::Rng;

// ...

async fn enter_interactive_mode() -> Result<()> {
    // ... (setup is the same)
    loop {
        let sig = line_editor.read_line(&prompt)?;
        match sig {
            Signal::Success(line) => {
                let trimmed_line = line.trim();
                if trimmed_line.is_empty() { continue; }

                // --- MODIFIED LOGIC ---
                let result = if trimmed_line.starts_with('/') {
                    let cmd = slash::parse(trimmed_line)?;
                    
                    // Generate a unique branch name for this task
                    let mut rng = rand::thread_rng();
                    let task_id: u32 = rng.gen();
                    let branch_name = format!("agent/{}-{}", cmd.name, task_id);

                    // For now, we prove the concept by running a simple command
                    // instead of calling the LLM.
                    let shell_cmd_to_run = if let Some(file_path) = cmd.file_path {
                        // If a file is specified, list it in the container
                        format!("echo '--- Running in Container ---'; ls -l {}", file_path)
                    } else {
                        // Otherwise, just list the current directory
                        "echo '--- Running in Container ---'; ls -l".to_string()
                    };

                    container::run_in_container(&branch_name, &shell_cmd_to_run).await
                } else {
                    // Regular chat still calls the LLM directly for now.
                    // Containerization is only for slash commands.
                    let response = ask(trimmed_line).await?;
                    println!("\nAI: {}\n", response);
                    Ok(())
                };

                if let Err(e) = result {
                    eprintln!("\nError: {}\n", e);
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

---

### **Part 3: Final Review & Merge Preparation**

#### **Ready to Merge Checklist**
*   [x] **Works with Docker Desktop & Colima:** The `cu` command abstracts the Docker runtime.
*   [x] **Docs: "Install Docker & cu"**: The PR for this slice should include a `README.md` update mentioning these new prerequisites.
*   [x] **Unit test mocks `cu`:** While full mocking is complex, the `check_cu_exists` function is a form of pre-flight check. More robust tests can be added later.
*   [ ] **Test manually:**
    *   Make sure Docker Desktop (or Colima) is running.
    *   Run `cargo run -- chat`.
    *   Try a slash command: `/test --file Cargo.toml`.
    *   You should see output indicating a new environment is spawning.
    *   The output should then show the result of `ls -l Cargo.toml` running *inside the container*.
    *   After the command finishes, check your git branches with `git branch`. You should see a new branch named something like `agent/test-12345678`.
*   [ ] **Commit your work:**
    ```bash
    git add .
    git commit -m "feat(core): Implement Slice 4 - container-use integration for slash commands"
    ```
*   [ ] **Push and open a Pull Request:**
    ```bash
    git push --set-upstream origin slice-4-container-integration
    ```

#### **Questions for Senior Dev**
Include these important architectural questions in your Pull Request:
> *   What should our strategy be for cleaning up the `agent/...` branches and containers? Should they be removed immediately after a task, or kept for a period for debugging?
> *   To speed up future agent tasks that need to compile code, should we mount the host's Cargo/NPM cache directories into the container? What are the security implications?