use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub persona: String,
    pub status: AgentStatus,
    pub branch_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum AgentStatus {
    Running,
    Stopped,
    Error(String),
}

pub struct AgentSupervisor {
    agents: Arc<Mutex<HashMap<String, Agent>>>,
}

impl AgentSupervisor {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn spawn(&mut self, id: &str, persona: &str) -> Result<()> {
        let mut agents = self.agents.lock().await;
        
        if agents.contains_key(id) {
            return Err(anyhow::anyhow!("Agent with id '{}' already exists", id));
        }

        let agent = Agent {
            id: id.to_string(),
            persona: persona.to_string(),
            status: AgentStatus::Running,
            branch_name: format!("agent-{}", id),
        };

        agents.insert(id.to_string(), agent);
        Ok(())
    }

    pub async fn list(&self) -> Vec<Agent> {
        let agents = self.agents.lock().await;
        agents.values().cloned().collect()
    }

    pub async fn stop(&mut self, id: &str) -> Result<()> {
        let mut agents = self.agents.lock().await;
        
        let agent = agents.get_mut(id)
            .context(format!("Agent '{}' not found", id))?;
        
        agent.status = AgentStatus::Stopped;
        Ok(())
    }

    pub async fn get_status(&self, id: &str) -> Result<AgentStatus> {
        let agents = self.agents.lock().await;
        
        let agent = agents.get(id)
            .context(format!("Agent '{}' not found", id))?;
        
        Ok(agent.status.clone())
    }
}

impl Default for AgentSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_supervisor_new() {
        let supervisor = AgentSupervisor::new();
        let agents = supervisor.list().await;
        assert_eq!(agents.len(), 0);
    }

    #[tokio::test]
    async fn test_spawn_agent() {
        let mut supervisor = AgentSupervisor::new();
        let result = supervisor.spawn("test-agent", "rusty").await;
        assert!(result.is_ok());

        let agents = supervisor.list().await;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].id, "test-agent");
        assert_eq!(agents[0].persona, "rusty");
    }

    #[tokio::test]
    async fn test_spawn_duplicate_agent() {
        let mut supervisor = AgentSupervisor::new();
        supervisor.spawn("test-agent", "rusty").await.unwrap();
        
        let result = supervisor.spawn("test-agent", "pythonic").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stop_agent() {
        let mut supervisor = AgentSupervisor::new();
        supervisor.spawn("test-agent", "rusty").await.unwrap();
        
        let result = supervisor.stop("test-agent").await;
        assert!(result.is_ok());

        let agents = supervisor.list().await;
        assert!(matches!(agents[0].status, AgentStatus::Stopped));
    }

    #[tokio::test]
    async fn test_get_status() {
        let mut supervisor = AgentSupervisor::new();
        supervisor.spawn("test-agent", "rusty").await.unwrap();
        
        let status = supervisor.get_status("test-agent").await.unwrap();
        assert!(matches!(status, AgentStatus::Running));
        
        supervisor.stop("test-agent").await.unwrap();
        let status = supervisor.get_status("test-agent").await.unwrap();
        assert!(matches!(status, AgentStatus::Stopped));
    }

    #[tokio::test]
    async fn test_get_status_nonexistent() {
        let supervisor = AgentSupervisor::new();
        let result = supervisor.get_status("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_spawn_multiple_agents() {
        let mut supervisor = AgentSupervisor::new();
        
        supervisor.spawn("agent1", "rusty").await.unwrap();
        supervisor.spawn("agent2", "pythonic").await.unwrap();
        
        let agents = supervisor.list().await;
        assert_eq!(agents.len(), 2);
        
        let agent1 = agents.iter().find(|a| a.id == "agent1").unwrap();
        let agent2 = agents.iter().find(|a| a.id == "agent2").unwrap();
        
        assert_eq!(agent1.persona, "rusty");
        assert_eq!(agent2.persona, "pythonic");
        assert!(matches!(agent1.status, AgentStatus::Running));
        assert!(matches!(agent2.status, AgentStatus::Running));
    }

    #[tokio::test]
    async fn test_stop_nonexistent_agent() {
        let mut supervisor = AgentSupervisor::new();
        let result = supervisor.stop("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_status_serialization() {
        let running = AgentStatus::Running;
        let stopped = AgentStatus::Stopped;
        let error = AgentStatus::Error("test error".to_string());
        
        let running_json = serde_json::to_string(&running).unwrap();
        let stopped_json = serde_json::to_string(&stopped).unwrap();
        let error_json = serde_json::to_string(&error).unwrap();
        
        assert_eq!(running_json, "\"Running\"");
        assert_eq!(stopped_json, "\"Stopped\"");
        assert!(error_json.contains("test error"));
    }

    #[tokio::test]
    async fn test_concurrent_agent_operations() {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        
        let supervisor = Arc::new(Mutex::new(AgentSupervisor::new()));
        let mut handles = vec![];
        
        // Spawn 10 agents concurrently
        for i in 0..10 {
            let supervisor = supervisor.clone();
            let handle = tokio::spawn(async move {
                let mut sup = supervisor.lock().await;
                sup.spawn(&format!("agent{}", i), "rusty").await
            });
            handles.push(handle);
        }
        
        // Wait for all spawn operations to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }
        
        let supervisor = supervisor.lock().await;
        let agents = supervisor.list().await;
        assert_eq!(agents.len(), 10);
    }
}