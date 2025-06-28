use crate::config::Config;
use crate::error::{Error, Result};
use crate::supervisor::{Supervisor, AgentStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

/// Swarm orchestrator that manages multiple supervisors and coordinates agent swarms
#[derive(Debug)]
pub struct SwarmOrchestrator {
    config: Config,
    supervisors: Arc<RwLock<HashMap<String, Arc<Supervisor>>>>,
    started_at: Instant,
}

#[derive(Debug, Clone)]
pub struct SwarmInfo {
    pub id: String,
    pub supervisor_count: usize,
    pub total_agents: usize,
    pub active_agents: usize,
    pub status: SwarmStatus,
    pub created_at: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SwarmStatus {
    Initializing,
    Active,
    Scaling,
    Degraded,
    Shutdown,
}

#[derive(Debug)]
pub struct SwarmMetrics {
    pub total_supervisors: usize,
    pub total_agents: usize,
    pub active_agents: usize,
    pub failed_agents: usize,
    pub tasks_processed: usize,
    pub uptime: Duration,
    pub memory_usage: usize,
}

impl SwarmOrchestrator {
    /// Create a new swarm orchestrator
    pub fn new(config: Config) -> Self {
        Self {
            config,
            supervisors: Arc::new(RwLock::new(HashMap::new())),
            started_at: Instant::now(),
        }
    }

    /// Check if the swarm orchestrator is healthy
    pub async fn is_healthy(&self) -> bool {
        let supervisors = self.supervisors.read().await;
        
        if supervisors.is_empty() {
            return false;
        }

        // Check if at least one supervisor is healthy
        for supervisor in supervisors.values() {
            if let Ok(health) = supervisor.health_check().await {
                if health.is_healthy {
                    return true;
                }
            }
        }

        false
    }

    /// Add a supervisor to the swarm
    pub async fn add_supervisor(&self, supervisor_id: String, supervisor: Arc<Supervisor>) -> Result<()> {
        let mut supervisors = self.supervisors.write().await;
        
        if supervisors.contains_key(&supervisor_id) {
            return Err(Error::Service(format!("Supervisor {} already exists", supervisor_id)));
        }

        supervisors.insert(supervisor_id, supervisor);
        Ok(())
    }

    /// Remove a supervisor from the swarm
    pub async fn remove_supervisor(&self, supervisor_id: &str) -> Result<()> {
        let mut supervisors = self.supervisors.write().await;
        
        match supervisors.remove(supervisor_id) {
            Some(supervisor) => {
                // Gracefully shutdown the supervisor
                supervisor.shutdown().await?;
                Ok(())
            }
            None => Err(Error::Service(format!("Supervisor {} not found", supervisor_id))),
        }
    }

    /// Get a supervisor by ID
    pub async fn get_supervisor(&self, supervisor_id: &str) -> Result<Arc<Supervisor>> {
        let supervisors = self.supervisors.read().await;
        supervisors.get(supervisor_id)
            .cloned()
            .ok_or_else(|| Error::Service(format!("Supervisor {} not found", supervisor_id)))
    }

    /// List all supervisors in the swarm
    pub async fn list_supervisors(&self) -> Vec<String> {
        let supervisors = self.supervisors.read().await;
        supervisors.keys().cloned().collect()
    }

    /// Get comprehensive swarm metrics
    pub async fn get_metrics(&self) -> SwarmMetrics {
        let supervisors = self.supervisors.read().await;
        let mut total_agents = 0;
        let mut active_agents = 0;
        let mut failed_agents = 0;
        let mut tasks_processed = 0;
        let mut memory_usage = 0;

        for supervisor in supervisors.values() {
            if let Ok(health) = supervisor.health_check().await {
                total_agents += health.total_agents;
                active_agents += health.running_agents;
                failed_agents += health.failed_agents;
                memory_usage += health.memory_usage;
            }

            let stats = supervisor.get_stats().await;
            tasks_processed += stats.total_tasks;
        }

        SwarmMetrics {
            total_supervisors: supervisors.len(),
            total_agents,
            active_agents,
            failed_agents,
            tasks_processed,
            uptime: self.started_at.elapsed(),
            memory_usage,
        }
    }

    /// Scale the swarm by adding agents to supervisors
    pub async fn scale_up(&self, target_agents_per_supervisor: usize) -> Result<()> {
        let supervisors = self.supervisors.read().await;
        
        for (supervisor_id, supervisor) in supervisors.iter() {
            let current_agents = supervisor.list_agents().await.len();
            
            if current_agents < target_agents_per_supervisor {
                let agents_to_add = target_agents_per_supervisor - current_agents;
                
                for i in 0..agents_to_add {
                    let agent_id = format!("{}-agent-{}", supervisor_id, current_agents + i + 1);
                    supervisor.register_agent(agent_id).await?;
                }
            }
        }

        Ok(())
    }

    /// Scale down the swarm by removing agents
    pub async fn scale_down(&self, target_agents_per_supervisor: usize) -> Result<()> {
        let supervisors = self.supervisors.read().await;
        
        for supervisor in supervisors.values() {
            let agents = supervisor.list_agents().await;
            
            if agents.len() > target_agents_per_supervisor {
                let agents_to_remove = agents.len() - target_agents_per_supervisor;
                
                // Remove idle agents first
                let mut removed = 0;
                for agent in agents.iter() {
                    if removed >= agents_to_remove {
                        break;
                    }
                    
                    if agent.status == AgentStatus::Idle {
                        supervisor.unregister_agent(&agent.id).await?;
                        removed += 1;
                    }
                }
            }
        }

        Ok(())
    }

    /// Rebalance agents across supervisors
    pub async fn rebalance(&self) -> Result<()> {
        let supervisors = self.supervisors.read().await;
        
        if supervisors.len() < 2 {
            return Ok(()) // Nothing to rebalance
        }

        // Calculate total agents and target per supervisor
        let mut total_agents = 0;
        for supervisor in supervisors.values() {
            total_agents += supervisor.list_agents().await.len();
        }

        let target_per_supervisor = total_agents / supervisors.len();
        let remainder = total_agents % supervisors.len();

        // For simplicity, this is a basic rebalancing strategy
        // In a real implementation, you'd want more sophisticated load balancing
        
        for (i, (_supervisor_id, supervisor)) in supervisors.iter().enumerate() {
            let current_agents = supervisor.list_agents().await.len();
            let target = if i < remainder { target_per_supervisor + 1 } else { target_per_supervisor };
            
            if current_agents > target {
                let excess = current_agents - target;
                // Remove excess agents (in real implementation, migrate to other supervisors)
                let agents = supervisor.list_agents().await;
                for agent in agents.iter().take(excess) {
                    if agent.status == AgentStatus::Idle {
                        supervisor.unregister_agent(&agent.id).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Perform health checks on all supervisors and recover failed ones
    pub async fn health_check_and_recover(&self) -> Result<Vec<String>> {
        let supervisors = self.supervisors.read().await;
        let mut recovered_supervisors = Vec::new();

        for (supervisor_id, supervisor) in supervisors.iter() {
            match supervisor.health_check().await {
                Ok(health) => {
                    if !health.is_healthy && health.failed_agents > 0 {
                        // Attempt to recover failed agents
                        let agents = supervisor.list_agents().await;
                        for agent in agents.iter() {
                            if agent.status == AgentStatus::Failed {
                                // In a real implementation, this would restart the agent
                                supervisor.update_agent_status(&agent.id, AgentStatus::Starting).await?;
                            }
                        }
                        recovered_supervisors.push(supervisor_id.clone());
                    }
                }
                Err(_) => {
                    // Supervisor is completely unresponsive
                    // In a real implementation, you might restart the supervisor
                    recovered_supervisors.push(supervisor_id.clone());
                }
            }
        }

        Ok(recovered_supervisors)
    }

    /// Shutdown the entire swarm
    pub async fn shutdown(&self) -> Result<()> {
        let supervisors = self.supervisors.read().await;
        
        for supervisor in supervisors.values() {
            supervisor.shutdown().await?;
        }

        Ok(())
    }

    /// Get swarm information
    pub async fn get_swarm_info(&self) -> SwarmInfo {
        let supervisors = self.supervisors.read().await;
        let mut total_agents = 0;
        let mut active_agents = 0;

        for supervisor in supervisors.values() {
            let agents = supervisor.list_agents().await;
            total_agents += agents.len();
            active_agents += agents.iter()
                .filter(|a| a.status == AgentStatus::Running || a.status == AgentStatus::Busy)
                .count();
        }

        let status = if supervisors.is_empty() {
            SwarmStatus::Shutdown
        } else if active_agents == 0 {
            SwarmStatus::Degraded
        } else {
            SwarmStatus::Active
        };

        SwarmInfo {
            id: "main-swarm".to_string(),
            supervisor_count: supervisors.len(),
            total_agents,
            active_agents,
            status,
            created_at: self.started_at,
        }
    }

    /// Monitor swarm and auto-scale based on load
    pub async fn auto_scale(&self, min_agents_per_supervisor: usize, max_agents_per_supervisor: usize) -> Result<()> {
        let supervisors = self.supervisors.read().await;
        
        for supervisor in supervisors.values() {
            let agents = supervisor.list_agents().await;
            let busy_agents = agents.iter()
                .filter(|a| a.status == AgentStatus::Busy)
                .count();
            let total_agents = agents.len();

            // Scale up if more than 80% of agents are busy
            if total_agents > 0 && (busy_agents as f64 / total_agents as f64) > 0.8 && total_agents < max_agents_per_supervisor {
                let agent_id = format!("auto-scale-agent-{}", total_agents + 1);
                supervisor.register_agent(agent_id).await?;
            }
            // Scale down if less than 20% of agents are busy
            else if total_agents > min_agents_per_supervisor && (busy_agents as f64 / total_agents as f64) < 0.2 {
                // Find an idle agent to remove
                for agent in agents.iter() {
                    if agent.status == AgentStatus::Idle {
                        supervisor.unregister_agent(&agent.id).await?;
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_swarm_orchestrator_creation() {
        let config = Config::default();
        let orchestrator = SwarmOrchestrator::new(config);
        
        assert!(!orchestrator.is_healthy().await);
        
        let info = orchestrator.get_swarm_info().await;
        assert_eq!(info.supervisor_count, 0);
        assert_eq!(info.status, SwarmStatus::Shutdown);
    }

    #[tokio::test]
    async fn test_add_remove_supervisor() {
        let config = Config::default();
        let orchestrator = SwarmOrchestrator::new(config.clone());
        
        let supervisor = Arc::new(Supervisor::new(config));
        orchestrator.add_supervisor("test-supervisor".to_string(), supervisor).await.unwrap();
        
        let supervisors = orchestrator.list_supervisors().await;
        assert_eq!(supervisors.len(), 1);
        assert_eq!(supervisors[0], "test-supervisor");
        
        orchestrator.remove_supervisor("test-supervisor").await.unwrap();
        let supervisors = orchestrator.list_supervisors().await;
        assert_eq!(supervisors.len(), 0);
    }

    #[tokio::test]
    async fn test_swarm_metrics() {
        let config = Config::default();
        let orchestrator = SwarmOrchestrator::new(config.clone());
        
        let supervisor = Arc::new(Supervisor::new(config));
        supervisor.register_agent("test-agent".to_string()).await.unwrap();
        supervisor.update_agent_status("test-agent", AgentStatus::Running).await.unwrap();
        
        orchestrator.add_supervisor("test-supervisor".to_string(), supervisor).await.unwrap();
        
        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_supervisors, 1);
        assert_eq!(metrics.total_agents, 1);
        assert_eq!(metrics.active_agents, 1);
    }

    #[tokio::test]
    async fn test_scale_up() {
        let config = Config::default();
        let orchestrator = SwarmOrchestrator::new(config.clone());
        
        let supervisor = Arc::new(Supervisor::new(config));
        orchestrator.add_supervisor("test-supervisor".to_string(), supervisor.clone()).await.unwrap();
        
        orchestrator.scale_up(3).await.unwrap();
        
        let agents = supervisor.list_agents().await;
        assert_eq!(agents.len(), 3);
    }

    #[tokio::test]
    async fn test_scale_down() {
        let config = Config::default();
        let orchestrator = SwarmOrchestrator::new(config.clone());
        
        let supervisor = Arc::new(Supervisor::new(config));
        
        // Add some agents first
        for i in 0..5 {
            let agent_id = format!("agent-{}", i);
            supervisor.register_agent(agent_id.clone()).await.unwrap();
            supervisor.update_agent_status(&agent_id, AgentStatus::Idle).await.unwrap();
        }
        
        orchestrator.add_supervisor("test-supervisor".to_string(), supervisor.clone()).await.unwrap();
        
        orchestrator.scale_down(2).await.unwrap();
        
        let agents = supervisor.list_agents().await;
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn test_health_check_and_recover() {
        let config = Config::default();
        let orchestrator = SwarmOrchestrator::new(config.clone());
        
        let supervisor = Arc::new(Supervisor::new(config));
        supervisor.register_agent("test-agent".to_string()).await.unwrap();
        supervisor.update_agent_status("test-agent", AgentStatus::Failed).await.unwrap();
        
        orchestrator.add_supervisor("test-supervisor".to_string(), supervisor.clone()).await.unwrap();
        
        let recovered = orchestrator.health_check_and_recover().await.unwrap();
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0], "test-supervisor");
        
        // Check that agent status was updated
        let agent = supervisor.get_agent("test-agent").await.unwrap();
        assert_eq!(agent.status, AgentStatus::Starting);
    }

    #[tokio::test]
    async fn test_swarm_shutdown() {
        let config = Config::default();
        let orchestrator = SwarmOrchestrator::new(config.clone());
        
        let supervisor = Arc::new(Supervisor::new(config));
        supervisor.register_agent("test-agent".to_string()).await.unwrap();
        supervisor.update_agent_status("test-agent", AgentStatus::Running).await.unwrap();
        
        orchestrator.add_supervisor("test-supervisor".to_string(), supervisor.clone()).await.unwrap();
        
        orchestrator.shutdown().await.unwrap();
        
        let agent = supervisor.get_agent("test-agent").await.unwrap();
        assert_eq!(agent.status, AgentStatus::Stopped);
    }
}