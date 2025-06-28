use crate::container::ContainerManager;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
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