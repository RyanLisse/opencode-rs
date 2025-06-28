use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

#[cfg(test)]
use mockall::automock;

/// Trait for executing commands - allows mocking in tests
#[cfg_attr(test, automock)]
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, program: String, args: Vec<String>) -> Result<CommandOutput>;
    async fn spawn_command(&self, program: String, args: Vec<String>) -> Result<bool>;
}

pub struct CommandOutput {
    pub status: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Real implementation of CommandExecutor
pub struct RealCommandExecutor;

#[async_trait]
impl CommandExecutor for RealCommandExecutor {
    async fn execute(&self, program: String, args: Vec<String>) -> Result<CommandOutput> {
        let output = Command::new(&program)
            .args(&args)
            .output()
            .await
            .with_context(|| format!("Failed to execute '{}' command", program))?;

        Ok(CommandOutput {
            status: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    async fn spawn_command(&self, program: String, args: Vec<String>) -> Result<bool> {
        let mut child = Command::new(&program)
            .args(&args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to spawn '{}' process", program))?;

        let status = child
            .wait()
            .await
            .context("Failed to wait for process")?;

        Ok(status.success())
    }
}

/// Container manager that uses dependency injection for testability
pub struct ContainerManager {
    executor: Box<dyn CommandExecutor>,
}

impl ContainerManager {
    pub fn new(executor: Box<dyn CommandExecutor>) -> Self {
        Self { executor }
    }

    /// Checks if the 'cu' command is available on the system PATH.
    pub async fn check_cu_exists(&self) -> Result<()> {
        let output = self.executor.execute(
            "cu".to_string(), 
            vec!["--version".to_string()]
        ).await
        .context("Failed to execute 'cu' command. Is `container-use` installed and in your PATH?")?;

        if !output.status {
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
    pub async fn run_in_container(&self, branch_name: &str, shell_command: &str) -> Result<()> {
        println!(
            "\n--- Spawning environment for branch '{}' ---",
            branch_name
        );

        let args = vec![
            "environment".to_string(),
            "open".to_string(),
            "--branch".to_string(),
            branch_name.to_string(),
            "--".to_string(),
            "sh".to_string(),
            "-c".to_string(),
            shell_command.to_string(),
        ];

        let success = self.executor.spawn_command("cu".to_string(), args).await
            .context("Failed to spawn 'cu environment open' process.")?;

        println!("--- Environment for '{}' finished ---", branch_name);

        if !success {
            bail!("Container command exited with a non-zero status");
        }

        Ok(())
    }
}

/// Factory function for production use
pub fn create_container_manager() -> ContainerManager {
    ContainerManager::new(Box::new(RealCommandExecutor))
}

/// Convenience functions that use the default container manager
pub async fn check_cu_exists() -> Result<()> {
    create_container_manager().check_cu_exists().await
}

pub async fn run_in_container(branch_name: &str, shell_command: &str) -> Result<()> {
    create_container_manager().run_in_container(branch_name, shell_command).await
}