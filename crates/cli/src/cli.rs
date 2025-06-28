use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use opencode_core::supervisor::Supervisor;
use tracing::{info, error};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Agent management commands
    #[command(subcommand)]
    Agent(AgentCommands),
    
    /// Ask a question directly
    Ask {
        /// The question to ask
        question: String,
        
        /// Persona to use for the response
        #[arg(short, long, default_value = "default")]
        persona: String,
    },
    
    /// Start interactive REPL mode
    Repl,
    
    /// Show version information
    Version,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AgentCommands {
    /// List all running agents
    Ls,
    
    /// Spawn a new agent
    Spawn {
        /// Agent identifier
        id: String,
        
        /// Agent persona
        #[arg(short, long, default_value = "rusty")]
        persona: String,
    },
    
    /// Stop an agent
    Stop {
        /// Agent identifier
        id: String,
    },
    
    /// Get agent status
    Status {
        /// Agent identifier
        id: String,
    },
}

pub async fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::Agent(agent_cmd) => execute_agent_command(agent_cmd).await,
        Commands::Ask { question, persona } => execute_ask_command(&question, &persona).await,
        Commands::Repl => {
            // This should not happen in practice since None case goes to REPL
            // But we handle it for completeness
            crate::repl::start().await
        },
        Commands::Version => {
            execute_version_command().await
        },
    }
}

async fn execute_agent_command(command: AgentCommands) -> Result<()> {
    let mut supervisor = AgentSupervisor::new();
    
    match command {
        AgentCommands::Ls => {
            info!("Listing all agents");
            let agents = supervisor.list().await;
            if agents.is_empty() {
                println!("No agents running.");
            } else {
                println!("Running agents:");
                for agent in agents {
                    println!("  {} ({}): {:?}", agent.id, agent.persona, agent.status);
                }
            }
        }
        AgentCommands::Spawn { id, persona } => {
            info!("Spawning agent '{}' with persona '{}'", id, persona);
            supervisor.spawn(&id, &persona).await
                .with_context(|| format!("Failed to spawn agent '{}' with persona '{}'", id, persona))?;
            println!("Spawned agent '{}' with persona '{}'", id, persona);
        }
        AgentCommands::Stop { id } => {
            info!("Stopping agent '{}'", id);
            supervisor.stop(&id).await
                .with_context(|| format!("Failed to stop agent '{}'", id))?;
            println!("Stopped agent '{}'", id);
        }
        AgentCommands::Status { id } => {
            info!("Getting status for agent '{}'", id);
            match supervisor.get_status(&id).await {
                Ok(status) => println!("Agent '{}' status: {:?}", id, status),
                Err(e) => {
                    error!("Failed to get status for agent '{}': {}", id, e);
                    return Err(e.into());
                }
            }
        }
    }
    
    Ok(())
}

async fn execute_ask_command(question: &str, persona: &str) -> Result<()> {
    info!("Asking question with persona '{}'", persona);
    
    // Import ask function from core
    use opencode_core::ask_with_persona;
    
    match ask_with_persona(question, persona).await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            error!("Failed to get response: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn execute_version_command() -> Result<()> {
    println!("OpenCode-RS CLI v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use pretty_assertions::assert_eq;
    use test_case::test_case;

    #[test]
    fn test_cli_structure() {
        // Test that CLI can be built without errors
        let _cmd = Cli::command();
    }

    #[test]
    fn test_cli_help() {
        let mut cmd = Cli::command();
        let help = cmd.render_help();
        assert!(help.to_string().contains("OpenCode"));
    }

    #[test_case("agent ls"; "agent ls command")]
    #[test_case("ask What is Rust?"; "ask command")]
    #[test_case("version"; "version command")]
    fn test_command_parsing(cmd_line: &str) {
        let mut cmd_args = vec!["opencode"];
        cmd_args.extend(cmd_line.split_whitespace());
        
        let result = Cli::try_parse_from(cmd_args);
        assert!(result.is_ok(), "Failed to parse command: {}", cmd_line);
    }

    #[test]
    fn test_agent_spawn_parsing() {
        let cli = Cli::try_parse_from(["opencode", "agent", "spawn", "test-agent", "--persona", "test-persona"]).unwrap();
        
        match cli.command {
            Some(Commands::Agent(AgentCommands::Spawn { id, persona })) => {
                assert_eq!(id, "test-agent");
                assert_eq!(persona, "test-persona");
            }
            _ => panic!("Expected agent spawn command"),
        }
    }

    #[test]
    fn test_ask_command_parsing() {
        let cli = Cli::try_parse_from(["opencode", "ask", "What is Rust?", "--persona", "expert"]).unwrap();
        
        match cli.command {
            Some(Commands::Ask { question, persona }) => {
                assert_eq!(question, "What is Rust?");
                assert_eq!(persona, "expert");
            }
            _ => panic!("Expected ask command"),
        }
    }

    #[test]
    fn test_default_persona() {
        let cli = Cli::try_parse_from(["opencode", "ask", "What is Rust?"]).unwrap();
        
        match cli.command {
            Some(Commands::Ask { persona, .. }) => {
                assert_eq!(persona, "default");
            }
            _ => panic!("Expected ask command"),
        }
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::try_parse_from(["opencode", "--verbose", "version"]).unwrap();
        assert!(cli.verbose);
    }

    #[test]
    fn test_config_option() {
        let cli = Cli::try_parse_from(["opencode", "--config", "test.toml", "version"]).unwrap();
        assert_eq!(cli.config, Some("test.toml".to_string()));
    }

    #[test]
    fn test_invalid_command() {
        let result = Cli::try_parse_from(["opencode", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_commands_completeness() {
        // Ensure all agent commands are tested
        let cli = Cli::try_parse_from(["opencode", "agent", "ls"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Agent(AgentCommands::Ls))));
        
        let cli = Cli::try_parse_from(["opencode", "agent", "spawn", "test"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Agent(AgentCommands::Spawn { .. }))));
        
        let cli = Cli::try_parse_from(["opencode", "agent", "stop", "test"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Agent(AgentCommands::Stop { .. }))));
        
        let cli = Cli::try_parse_from(["opencode", "agent", "status", "test"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Agent(AgentCommands::Status { .. }))));
    }

    #[tokio::test]
    async fn test_version_command_execution() {
        let result = execute_version_command().await;
        assert!(result.is_ok());
    }

    // Property-based testing for command parsing
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_agent_id_parsing(id in "[a-zA-Z][a-zA-Z0-9_-]*") {
                let args = vec!["opencode", "agent", "spawn", &id];
                let result = Cli::try_parse_from(args);
                prop_assert!(result.is_ok());
            }

            #[test]
            fn test_ask_question_parsing(question in ".{1,100}") {
                let args = vec!["opencode", "ask", &question];
                let result = Cli::try_parse_from(args);
                prop_assert!(result.is_ok());
            }
        }
    }
}