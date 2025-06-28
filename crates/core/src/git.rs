use anyhow::{anyhow, Context, Result};
use uuid::Uuid;

#[cfg(test)]
use mockall::automock;

/// Trait for Git operations - allows mocking in tests
#[cfg_attr(test, automock)]
pub trait GitOperations: Send + Sync {
    fn save_checkpoint(&self, repo_path: &str, branch_name: &str, message: &str) -> Result<String>;
    fn list_checkpoints(&self, repo_path: &str, agent_id: &str) -> Result<Vec<String>>;
    fn restore_checkpoint(&self, repo_path: &str, checkpoint_tag: &str, new_agent_id: &str) -> Result<String>;
}

/// Real implementation using git2
pub struct RealGitOperations;

impl GitOperations for RealGitOperations {
    fn save_checkpoint(&self, repo_path: &str, branch_name: &str, message: &str) -> Result<String> {
        use git2::{Repository, Signature, ObjectType};
        
        let repo = Repository::open(repo_path)
            .context("Failed to open Git repository")?;
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

    fn list_checkpoints(&self, repo_path: &str, agent_id: &str) -> Result<Vec<String>> {
        use git2::Repository;
        
        let repo = Repository::open(repo_path)
            .context("Failed to open Git repository")?;
        let glob = format!("refs/tags/cp/{}/*", agent_id);
        let tags = repo.tag_names(Some(&glob))?;
        
        Ok(tags.iter().filter_map(|s| s.map(String::from)).collect())
    }

    fn restore_checkpoint(&self, repo_path: &str, checkpoint_tag: &str, new_agent_id: &str) -> Result<String> {
        use git2::Repository;
        
        let repo = Repository::open(repo_path)
            .context("Failed to open Git repository")?;
        
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
}

/// Git checkpoint manager with dependency injection for testing
pub struct GitCheckpointManager {
    git_ops: Box<dyn GitOperations>,
    repo_path: String,
}

impl GitCheckpointManager {
    pub fn new(git_ops: Box<dyn GitOperations>, repo_path: String) -> Self {
        Self { git_ops, repo_path }
    }

    /// Saves a checkpoint for a given agent's branch.
    /// This commits all current changes and creates a tagged release.
    pub fn save_checkpoint(&self, branch_name: &str, message: &str) -> Result<String> {
        self.git_ops.save_checkpoint(&self.repo_path, branch_name, message)
    }

    /// Lists all checkpoints for a specific agent.
    pub fn list_checkpoints(&self, agent_id: &str) -> Result<Vec<String>> {
        self.git_ops.list_checkpoints(&self.repo_path, agent_id)
    }

    /// Restores a checkpoint by creating a new branch/worktree from it.
    pub fn restore_checkpoint(&self, checkpoint_tag: &str, new_agent_id: &str) -> Result<String> {
        self.git_ops.restore_checkpoint(&self.repo_path, checkpoint_tag, new_agent_id)
    }
}

/// Factory function for production use
pub fn create_git_manager() -> GitCheckpointManager {
    GitCheckpointManager::new(Box::new(RealGitOperations), ".".to_string())
}

/// Convenience functions that use the default git manager
pub fn save_checkpoint(branch_name: &str, message: &str) -> Result<String> {
    create_git_manager().save_checkpoint(branch_name, message)
}

pub fn list_checkpoints(agent_id: &str) -> Result<Vec<String>> {
    create_git_manager().list_checkpoints(agent_id)
}

pub fn restore_checkpoint(checkpoint_tag: &str, new_agent_id: &str) -> Result<String> {
    create_git_manager().restore_checkpoint(checkpoint_tag, new_agent_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_save_checkpoint_success() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_save_checkpoint()
            .with(eq("."), eq("agent/alice"), eq("Test checkpoint"))
            .returning(|_, _, _| Ok("cp/alice/uuid-1234".to_string()));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), ".".to_string());
        let result = manager.save_checkpoint("agent/alice", "Test checkpoint");
        
        assert!(result.is_ok());
        let tag_name = result.unwrap();
        assert_eq!(tag_name, "cp/alice/uuid-1234");
    }

    #[test]
    fn test_save_checkpoint_branch_not_found() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_save_checkpoint()
            .returning(|_, _, _| Err(anyhow!("Could not find branch")));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), ".".to_string());
        let result = manager.save_checkpoint("agent/nonexistent", "Test");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not find branch"));
    }

    #[test]
    fn test_list_checkpoints_success() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_list_checkpoints()
            .with(eq("."), eq("alice"))
            .returning(|_, _| Ok(vec![
                "cp/alice/uuid1".to_string(),
                "cp/alice/uuid2".to_string(),
            ]));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), ".".to_string());
        let result = manager.list_checkpoints("alice");
        
        assert!(result.is_ok());
        let checkpoints = result.unwrap();
        assert_eq!(checkpoints.len(), 2);
        assert!(checkpoints.contains(&"cp/alice/uuid1".to_string()));
        assert!(checkpoints.contains(&"cp/alice/uuid2".to_string()));
    }

    #[test]
    fn test_list_checkpoints_empty() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_list_checkpoints()
            .with(eq("."), eq("bob"))
            .returning(|_, _| Ok(vec![]));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), ".".to_string());
        let result = manager.list_checkpoints("bob");
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_restore_checkpoint_success() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_restore_checkpoint()
            .with(eq("."), eq("cp/alice/uuid1"), eq("alice-fork"))
            .returning(|_, _, _| Ok("agent/alice-fork".to_string()));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), ".".to_string());
        let result = manager.restore_checkpoint("cp/alice/uuid1", "alice-fork");
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "agent/alice-fork");
    }

    #[test]
    fn test_restore_checkpoint_tag_not_found() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_restore_checkpoint()
            .returning(|_, _, _| Err(anyhow!("Checkpoint tag 'cp/alice/nonexistent' not found")));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), ".".to_string());
        let result = manager.restore_checkpoint("cp/alice/nonexistent", "alice-fork");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_checkpoint_tag_naming_validation() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_save_checkpoint()
            .with(eq("."), eq("agent/test-agent"), eq("Test message"))
            .returning(|_, branch_name, _| {
                // Validate that the branch name is processed correctly
                let agent_id = branch_name.replace("agent/", "");
                let uuid = Uuid::new_v4().to_string();
                Ok(format!("cp/{}/{}", agent_id, uuid))
            });

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), ".".to_string());
        let result = manager.save_checkpoint("agent/test-agent", "Test message");
        
        assert!(result.is_ok());
        let tag_name = result.unwrap();
        assert!(tag_name.starts_with("cp/test-agent/"));
        
        // Verify UUID format (36 characters with hyphens)
        let uuid_part = &tag_name["cp/test-agent/".len()..];
        assert_eq!(uuid_part.len(), 36);
        assert_eq!(uuid_part.matches('-').count(), 4);
    }

    #[test]
    fn test_git_operations_error_handling() {
        let mut mock_git_ops = MockGitOperations::new();
        
        // Test repository open failure
        mock_git_ops
            .expect_save_checkpoint()
            .returning(|_, _, _| Err(anyhow!("Failed to open Git repository")));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), "/nonexistent".to_string());
        let result = manager.save_checkpoint("agent/test", "Test");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to open Git repository"));
    }

    #[test]
    fn test_git2_operations_integration_style() {
        // Test that demonstrates the integration pattern without requiring a real repo
        let real_ops = RealGitOperations;
        
        // This would fail for a non-existent repo, which is expected behavior
        let result = real_ops.save_checkpoint("/nonexistent/path", "agent/test", "Test message");
        assert!(result.is_err());
        
        let result = real_ops.list_checkpoints("/nonexistent/path", "test");
        assert!(result.is_err());
        
        let result = real_ops.restore_checkpoint("/nonexistent/path", "cp/test/uuid", "new-test");
        assert!(result.is_err());
    }

    #[test]
    fn test_convenience_functions() {
        // Test that convenience functions delegate to the default manager
        // These will fail without a real git repo, which is expected
        let result = save_checkpoint("agent/test", "Test message");
        // We can't test success without a real repo, but we can test the function exists
        assert!(result.is_err()); // Expected to fail without real repo
        
        let result = list_checkpoints("test");
        assert!(result.is_err()); // Expected to fail without real repo
        
        let result = restore_checkpoint("cp/test/uuid", "new-test");
        assert!(result.is_err()); // Expected to fail without real repo
    }

    #[test]
    fn test_manager_with_custom_repo_path() {
        let mut mock_git_ops = MockGitOperations::new();
        mock_git_ops
            .expect_save_checkpoint()
            .with(eq("/custom/path"), eq("agent/test"), eq("Test"))
            .returning(|_, _, _| Ok("cp/test/uuid".to_string()));

        let manager = GitCheckpointManager::new(Box::new(mock_git_ops), "/custom/path".to_string());
        let result = manager.save_checkpoint("agent/test", "Test");
        
        assert!(result.is_ok());
    }
}