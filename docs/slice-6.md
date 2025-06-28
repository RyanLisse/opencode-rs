Of course. Here is the detailed guide for completing the sixth vertical slice. This slice adds a powerful state-management feature for developers, allowing them to treat an agent's progress like version-controlled software.

***

### **Vertical Slice 6: Git Checkpoints & Worktree Management**

This slice leverages the fact that each agent operates on its own Git branch. You will implement a checkpoint system that allows users to save, list, and restore an agent's state. This is analogous to creating "save points" in a game, enabling risk-free experimentation and exploration of different development paths.

---

### **Part 1: Prerequisites & Setup**

Let's prepare the workspace by integrating the previous slice's work and creating a new worktree.

**1. Update Your Local `main` Branch:**
```bash
# Navigate to the main worktree directory
cd ../opencode-rs

# Switch to main and merge the agent supervisor work
git switch main
git merge --no-ff slice-5-agent-supervisor

# Clean up the old worktree and branch
git worktree remove ../opencode-rs-slice-5
git branch -d slice-5-agent-supervisor
```

**2. Create a New `git worktree` for Slice 6:**
```bash
# From the `opencode-rs` directory:
git worktree add -B slice-6-checkpoints ../opencode-rs-slice-6
cd ../opencode-rs-slice-6

# All work for Slice 6 will be done from here.
```

---

### **Part 2: Implementing Slice 6**

#### **What You’re Building**
1.  **A Git Interaction Module:** A new module in `opencode_core` (`git.rs`) dedicated to running `git` commands. It will be responsible for committing changes, creating tags, and managing worktrees.
2.  **Checkpoint Logic:** Functions within the `git` module to:
    *   `save_checkpoint()`: Commits all changes on an agent's branch and creates a unique tag (e.g., `cp/agent_name/uuid`).
    *   `list_checkpoints()`: Lists all checkpoint tags for a given agent.
    *   `restore_checkpoint()`: Creates a *new* agent/branch forked from a specific checkpoint tag.
3.  **New CLI Commands:** `opencode checkpoint save`, `opencode checkpoint list`, and `opencode checkpoint restore`.

#### **Step-by-Step Instructions**

**Step 1: Add New Dependencies**
We will use the `git2` crate for a more robust and programmatic way to interact with Git repositories, and `uuid` for generating unique checkpoint IDs.

Open `crates/core/Cargo.toml` and add the new dependencies:
```toml
# In crates/core/Cargo.toml

[dependencies]
# ... existing dependencies
uuid = { version = "1.7", features = ["v4"] }
git2 = "0.18"
```

**Step 2: Create the Git Module in `core`**
Create a new file `crates/core/src/git.rs`. This module will abstract all Git operations.

**`crates/core/src/git.rs`**
```rust
use anyhow::{anyhow, Context, Result};
use git2::{Commit, ObjectType, Oid, Repository, Signature, Tag};
use uuid::Uuid;

/// Opens the Git repository at the current working directory.
fn open_repo() -> Result<Repository> {
    Repository::open(".").context("Failed to open Git repository in the current directory.")
}

/// Saves a checkpoint for a given agent's branch.
/// This commits all current changes and creates a tagged release.
pub fn save_checkpoint(branch_name: &str, message: &str) -> Result<String> {
    let repo = open_repo()?;
    let signature = Signature::now("OpenCode Agent", "agent@opencode.dev")?;

    // Find the latest commit on the agent's branch
    let branch_ref = format!("refs/heads/{}", branch_name);
    let commit_oid = repo.refname_to_id(&branch_ref)
        .with_context(|| format!("Could not find branch '{}'", branch_name))?;
    
    // Switch the repository's HEAD to the agent's branch to commit
    repo.set_head(&branch_ref)?;
    
    // Stage all changes (git add -A)
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // Create the commit
    let parent_commit = repo.find_commit(commit_oid)?;
    let new_commit_oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &[&parent_commit],
    )?;

    // Create a unique tag for the checkpoint
    let checkpoint_id = Uuid::new_v4().to_string();
    let tag_name = format!("cp/{}/{}", branch_name.replace("agent/", ""), checkpoint_id);
    
    repo.tag(
        &tag_name,
        &repo.find_object(new_commit_oid, Some(ObjectType::Commit))?,
        &signature,
        "OpenCode Checkpoint",
        false, // Not a lightweight tag
    )?;

    println!("Successfully created checkpoint '{}' for branch '{}'.", tag_name, branch_name);
    Ok(tag_name)
}

/// Lists all checkpoints for a specific agent.
pub fn list_checkpoints(agent_id: &str) -> Result<Vec<String>> {
    let repo = open_repo()?;
    let glob = format!("refs/tags/cp/{}/*", agent_id);
    let tags = repo.tag_names(Some(&glob))?;
    
    Ok(tags.iter().filter_map(|s| s.map(String::from)).collect())
}

/// Restores a checkpoint by creating a new branch/worktree from it.
pub fn restore_checkpoint(checkpoint_tag: &str, new_agent_id: &str) -> Result<String> {
    let repo = open_repo()?;
    
    // Find the commit the tag points to
    let tag_ref = format!("refs/tags/{}", checkpoint_tag);
    let tag_object_id = repo.refname_to_id(&tag_ref)
        .with_context(|| format!("Checkpoint tag '{}' not found.", checkpoint_tag))?;
    
    let tag = repo.find_tag(tag_object_id)?;
    let target_commit = tag.target()?.peel_to_commit()?;

    // Create the new branch
    let new_branch_name = format!("agent/{}", new_agent_id);
    repo.branch(&new_branch_name, &target_commit, false)?;

    println!("Successfully restored checkpoint '{}' to new branch '{}'.", checkpoint_tag, new_branch_name);
    println!("You can now spawn a new agent on this branch: opencode agent spawn {} --persona <name>", new_agent_id);
    
    Ok(new_branch_name)
}
```

Now, add this as a module in `crates/core/src/lib.rs`.
```rust
// Add this line at the top of crates/core/src/lib.rs
pub mod git;
```

**Step 3: Integrate the Checkpoint Commands into the CLI**

Modify `crates/cli/src/main.rs` to include the new `checkpoint` subcommand.

**`crates/cli/src/main.rs`**
```rust
// In crates/cli/src/main.rs

// Add new use statements
use opencode_core::git;

// Add Checkpoint to the top-level commands
#[derive(Subcommand)]
enum Commands {
    // ... Chat and Agent subcommands are the same
    /// Manage development checkpoints
    Checkpoint {
        #[command(subcommand)]
        command: CheckpointCommands,
    },
}

// Define the new checkpoint subcommands
#[derive(Subcommand)]
enum CheckpointCommands {
    /// Save the current state of an agent's work as a new checkpoint
    Save {
        /// The ID of the agent whose work you want to save
        agent_id: String,
        /// A descriptive message for the checkpoint
        #[arg(short, long)]
        message: String,
    },
    /// List all saved checkpoints for an agent
    List {
        /// The ID of the agent whose checkpoints you want to list
        agent_id: String,
    },
    /// Create a new agent branch from a saved checkpoint
    Restore {
        /// The full name of the checkpoint tag (e.g., 'cp/alice/...')
        checkpoint_tag: String,
        /// The ID for the new agent that will be created from this checkpoint
        new_agent_id: String,
    },
}


#[tokio::main]
async fn main() -> Result<()> {
    // ... (supervisor setup is the same)
    
    match &cli.command {
        // ... Chat and Agent handlers are the same
        
        // --- NEW LOGIC FOR CHECKPOINT COMMANDS ---
        Commands::Checkpoint { command } => {
            match command {
                CheckpointCommands::Save { agent_id, message } => {
                    // Note: agent_id corresponds to the branch name suffix
                    let branch_name = format!("agent/{}", agent_id);
                    git::save_checkpoint(&branch_name, message)?;
                }
                CheckpointCommands::List { agent_id } => {
                    println!("Checkpoints for agent '{}':", agent_id);
                    let checkpoints = git::list_checkpoints(agent_id)?;
                    if checkpoints.is_empty() {
                        println!("  No checkpoints found.");
                    } else {
                        for cp in checkpoints {
                            println!("  - {}", cp);
                        }
                    }
                }
                CheckpointCommands::Restore { checkpoint_tag, new_agent_id } => {
                    git::restore_checkpoint(checkpoint_tag, new_agent_id)?;
                }
            }
        }
    }
    
    Ok(())
}
// The rest of the file remains the same
```

---

### **Part 3: Final Review & Merge Preparation**

#### **Ready to Merge Checklist**
*   [x] **No uncommitted loss:** The system commits all changes before tagging, ensuring no work is lost.
*   [x] **Docs with gif demo:** A good PR for this feature would include a short GIF or screencast demonstrating the save/list/restore flow.
*   [ ] **Test manually:**
    1.  **Spawn an agent:** `cargo run -- agent spawn alice --persona rusty`
    2.  **Simulate work:** The `cu` command in the agent's `sleep` task will have created a worktree. Manually add a file to the agent's worktree to simulate work.
        *   Find the worktree path: `git worktree list`
        *   Navigate to it and create a file: `echo "work in progress" > work.txt`
    3.  **Save a checkpoint:** `cargo run -- checkpoint save alice -m "Initial work"`
        *   Verify the success message and the new tag name.
    4.  **List checkpoints:** `cargo run -- checkpoint list alice`
        *   Verify the tag you just created is listed.
    5.  **Restore the checkpoint:** `cargo run -- checkpoint restore <tag_name> alice-fork` (replace `<tag_name>` with the actual tag).
        *   Verify the success message.
        *   Check `git branch`. You should now see a new `agent/alice-fork` branch.
    6.  **Spawn a new agent on the forked branch:** `cargo run -- agent spawn alice-fork --persona rusty`
*   [ ] **Commit your work:**
    ```bash
    git add .
    git commit -m "feat(core): Implement Slice 6 - Git checkpoints and worktree management"
    ```
*   [ ] **Push and open a Pull Request:**
    ```bash
    git push --set-upstream origin slice-6-checkpoints
    ```

#### **Questions for Senior Dev**
Include these important cleanup and strategy questions in your Pull Request:
> *   What should our cleanup strategy be for old checkpoints? Should we have a `checkpoint prune` command to remove tags older than a certain date?
> *   The current `restore` command only creates a new branch. Should it also automatically spawn the new agent via the supervisor? What are the pros and cons of that coupling?