use crate::container::ContainerManager;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Agent {
    pub id: String,
    pub persona: String,
    pub status: AgentStatus,
    pub branch_name: String,
}

/// Trait for persona validation - allows mocking in tests
pub trait PersonaValidator: Send + Sync {
    fn validate_persona(&self, persona_name: &str) -> Result<()>;
}

pub struct DefaultPersonaValidator;

impl PersonaValidator for DefaultPersonaValidator {
    fn validate_persona(&self, persona_name: &str) -> Result<()> {
        // In a real implementation, this would load from personas.yml
        // For now, we'll accept any non-empty persona name
        if persona_name.is_empty() {
            return Err(anyhow!("Persona name cannot be empty"));
        }
        
        // Mock validation for common personas
        match persona_name {
            "rusty" | "security-expert" | "architect" | "frontend" => Ok(()),
            _ => Err(anyhow!("Persona '{}' not found in personas.yml", persona_name))
        }
    }
}

// The supervisor manages all agents. It's designed to be thread-safe.
pub struct AgentSupervisor {
    agents: HashMap<String, Agent>,
    // This will hold the handles to the agent's background tasks
    tasks: HashMap<String, JoinHandle<()>>,
    container_manager: Arc<ContainerManager>,
    persona_validator: Box<dyn PersonaValidator>,
}

impl AgentSupervisor {
    pub fn new(
        container_manager: Arc<ContainerManager>,
        persona_validator: Box<dyn PersonaValidator>
    ) -> Self {
        Self {
            agents: HashMap::new(),
            tasks: HashMap::new(),
            container_manager,
            persona_validator,
        }
    }

    /// Spawns a new agent in a containerized environment.
    pub async fn spawn(&mut self, id: &str, persona_name: &str) -> Result<()> {
        if self.agents.contains_key(id) {
            return Err(anyhow!("Agent with ID '{}' already exists.", id));
        }

        // Validate that the persona exists
        self.persona_validator.validate_persona(persona_name)
            .with_context(|| format!("Invalid persona '{}'", persona_name))?;

        let branch_name = format!("agent/{}", id);

        let agent = Agent {
            id: id.to_string(),
            persona: persona_name.to_string(),
            status: AgentStatus::Running,
            branch_name: branch_name.clone(),
        };
        self.agents.insert(id.to_string(), agent.clone());

        // This is a placeholder for a long-running agent task.
        // In the future, this will be a loop listening for messages.
        // For now, it just keeps the agent "alive" for demonstration.
        let container_manager = Arc::clone(&self.container_manager);
        let agent_id = id.to_string();
        
        let task = tokio::spawn(async move {
            let shell_command = "echo 'Agent started. Waiting for tasks...'; sleep 3600";
            match container_manager.run_in_container(&branch_name, shell_command).await {
                Ok(_) => {
                    println!("Agent '{}' task finished.", agent_id);
                }
                Err(e) => {
                    eprintln!("Agent '{}' encountered an error: {}", agent_id, e);
                    // In a real implementation, we would update the agent's status here.
                }
            }
        });

        self.tasks.insert(id.to_string(), task);
        println!("Spawned agent '{}' with persona '{}'.", id, persona_name);
        Ok(())
    }

    /// Stops a running agent and cleans up its resources.
    pub async fn stop(&mut self, id: &str) -> Result<()> {
        let agent = self.agents.get_mut(id)
            .ok_or_else(|| anyhow!("Agent '{}' not found.", id))?;
        
        if let Some(task) = self.tasks.remove(id) {
            // Abort the background task. This will terminate the sleep command.
            task.abort();
            println!("Stopping agent '{}'...", id);

            // Here you would also add logic to stop the docker container.
            // For now, aborting the task is enough. `cu` might leave the container.
            // We will address container cleanup in a later slice.

            agent.status = AgentStatus::Stopped;
            println!("Agent '{}' stopped.", id);
            Ok(())
        } else {
            Err(anyhow!("Agent '{}' was not running.", id))
        }
    }

    /// Lists all agents managed by the supervisor.
    pub fn list(&self) -> Vec<Agent> {
        self.agents.values().cloned().collect()
    }

    /// Gets a specific agent by ID
    pub fn get(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    /// Updates an agent's status (useful for testing)
    pub fn update_agent_status(&mut self, id: &str, status: AgentStatus) -> Result<()> {
        let agent = self.agents.get_mut(id)
            .ok_or_else(|| anyhow!("Agent '{}' not found.", id))?;
        agent.status = status;
        Ok(())
    }

    /// Returns the number of running agents
    pub fn running_count(&self) -> usize {
        self.agents.values()
            .filter(|agent| matches!(agent.status, AgentStatus::Running))
            .count()
    }
}

/// Factory function for production use
pub fn create_supervisor(container_manager: Arc<ContainerManager>) -> AgentSupervisor {
    AgentSupervisor::new(container_manager, Box::new(DefaultPersonaValidator))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::MockCommandExecutor;
    use mockall::predicate::*;
    use std::sync::Arc;

    struct MockPersonaValidator {
        valid_personas: Vec<String>,
    }

    impl MockPersonaValidator {
        fn new(valid_personas: Vec<&str>) -> Self {
            Self {
                valid_personas: valid_personas.into_iter().map(String::from).collect(),
            }
        }
    }

    impl PersonaValidator for MockPersonaValidator {
        fn validate_persona(&self, persona_name: &str) -> Result<()> {
            if self.valid_personas.contains(&persona_name.to_string()) {
                Ok(())
            } else {
                Err(anyhow!("Persona '{}' not found", persona_name))
            }
        }
    }

    fn create_test_supervisor() -> (AgentSupervisor, Arc<ContainerManager>) {
        let mut mock_executor = MockCommandExecutor::new();
        mock_executor
            .expect_spawn_command()
            .returning(|_, _| Ok(true));

        let container_manager = Arc::new(ContainerManager::new(Box::new(mock_executor)));
        let persona_validator = Box::new(MockPersonaValidator::new(vec!["rusty", "security-expert"]));
        let supervisor = AgentSupervisor::new(container_manager.clone(), persona_validator);
        
        (supervisor, container_manager)
    }

    #[tokio::test]
    async fn test_spawn_agent_success() {
        let (mut supervisor, _) = create_test_supervisor();
        
        let result = supervisor.spawn("alice", "rusty").await;
        assert!(result.is_ok());
        
        let agents = supervisor.list();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "alice");
        assert_eq!(agents[0].persona, "rusty");
        assert_eq!(agents[0].status, AgentStatus::Running);
        assert_eq!(agents[0].branch_name, "agent/alice");
    }

    #[tokio::test]
    async fn test_spawn_agent_duplicate_id() {
        let (mut supervisor, _) = create_test_supervisor();
        
        // Spawn first agent
        supervisor.spawn("alice", "rusty").await.unwrap();
        
        // Try to spawn with same ID
        let result = supervisor.spawn("alice", "security-expert").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
        
        // Should still have only one agent
        assert_eq!(supervisor.list().len(), 1);
    }

    #[tokio::test]
    async fn test_spawn_agent_invalid_persona() {
        let (mut supervisor, _) = create_test_supervisor();
        
        let result = supervisor.spawn("alice", "invalid-persona").await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found") || error_msg.contains("Invalid persona"));
        
        // Should have no agents
        assert_eq!(supervisor.list().len(), 0);
    }

    #[tokio::test]
    async fn test_stop_agent_success() {
        let (mut supervisor, _) = create_test_supervisor();
        
        // Spawn an agent
        supervisor.spawn("alice", "rusty").await.unwrap();
        assert_eq!(supervisor.running_count(), 1);
        
        // Stop the agent
        let result = supervisor.stop("alice").await;
        assert!(result.is_ok());
        
        // Check status updated
        let agent = supervisor.get("alice").unwrap();
        assert_eq!(agent.status, AgentStatus::Stopped);
        assert_eq!(supervisor.running_count(), 0);
    }

    #[tokio::test]
    async fn test_stop_agent_not_found() {
        let (mut supervisor, _) = create_test_supervisor();
        
        let result = supervisor.stop("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_stop_agent_not_running() {
        let (mut supervisor, _) = create_test_supervisor();
        
        // Spawn and stop an agent
        supervisor.spawn("alice", "rusty").await.unwrap();
        supervisor.stop("alice").await.unwrap();
        
        // Try to stop again
        let result = supervisor.stop("alice").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
    }

    #[tokio::test]
    async fn test_concurrent_agent_spawning() {
        let (mut supervisor, _) = create_test_supervisor();
        
        // Spawn agents sequentially but verify they can be spawned concurrently
        // by testing the isolation of their configuration
        supervisor.spawn("alice", "rusty").await.unwrap();
        supervisor.spawn("bob", "security-expert").await.unwrap();
        
        let agents = supervisor.list();
        assert_eq!(agents.len(), 2);
        assert_eq!(supervisor.running_count(), 2);
        
        // Check that agents have different branches
        let alice = agents.iter().find(|a| a.id == "alice").unwrap();
        let bob = agents.iter().find(|a| a.id == "bob").unwrap();
        assert_eq!(alice.branch_name, "agent/alice");
        assert_eq!(bob.branch_name, "agent/bob");
        assert_ne!(alice.branch_name, bob.branch_name);
    }

    #[tokio::test]
    async fn test_agent_lifecycle_management() {
        let (mut supervisor, _) = create_test_supervisor();
        
        // Test full lifecycle
        supervisor.spawn("alice", "rusty").await.unwrap();
        supervisor.spawn("bob", "security-expert").await.unwrap();
        
        assert_eq!(supervisor.running_count(), 2);
        assert_eq!(supervisor.list().len(), 2);
        
        // Stop one agent
        supervisor.stop("alice").await.unwrap();
        assert_eq!(supervisor.running_count(), 1);
        assert_eq!(supervisor.list().len(), 2); // Still tracked but stopped
        
        // Stop the other
        supervisor.stop("bob").await.unwrap();
        assert_eq!(supervisor.running_count(), 0);
        assert_eq!(supervisor.list().len(), 2); // Still tracked but stopped
    }

    #[tokio::test]
    async fn test_thread_safe_state_management() {
        let (supervisor, _) = create_test_supervisor();
        let supervisor = Arc::new(Mutex::new(supervisor));
        
        // Simulate concurrent access
        let supervisor1 = Arc::clone(&supervisor);
        let supervisor2 = Arc::clone(&supervisor);
        
        let handle1 = tokio::spawn(async move {
            let mut sup = supervisor1.lock().await;
            sup.spawn("alice", "rusty").await
        });
        
        let handle2 = tokio::spawn(async move {
            let mut sup = supervisor2.lock().await;
            sup.spawn("bob", "security-expert").await
        });
        
        let (result1, result2) = tokio::join!(handle1, handle2);
        assert!(result1.unwrap().is_ok());
        assert!(result2.unwrap().is_ok());
        
        let sup = supervisor.lock().await;
        assert_eq!(sup.list().len(), 2);
    }

    #[tokio::test]
    async fn test_update_agent_status() {
        let (mut supervisor, _) = create_test_supervisor();
        
        supervisor.spawn("alice", "rusty").await.unwrap();
        
        // Update status to error
        let error_status = AgentStatus::Error("Test error".to_string());
        supervisor.update_agent_status("alice", error_status.clone()).unwrap();
        
        let agent = supervisor.get("alice").unwrap();
        assert_eq!(agent.status, error_status);
        assert_eq!(supervisor.running_count(), 0); // No longer running
    }

    #[tokio::test]
    async fn test_persona_validator() {
        let validator = DefaultPersonaValidator;
        
        // Test valid personas
        assert!(validator.validate_persona("rusty").is_ok());
        assert!(validator.validate_persona("security-expert").is_ok());
        
        // Test invalid personas
        assert!(validator.validate_persona("").is_err());
        assert!(validator.validate_persona("invalid").is_err());
    }
}