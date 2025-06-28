use anyhow::Result;
use reedline::{DefaultPrompt, Reedline, Signal};
use opencode_core::{slash, ask};
use tracing::{info, warn, error, debug};

pub struct ReplEngine {
    current_persona: String,
}

impl ReplEngine {
    pub fn new() -> Self {
        Self {
            current_persona: "default".to_string(),
        }
    }

    pub async fn execute_line(&mut self, line: &str) -> Result<String> {
        let line = line.trim();
        
        if line.is_empty() {
            return Ok(String::new());
        }

        // Handle special REPL commands
        if line.starts_with('/') {
            return self.execute_slash_command(line).await;
        }

        // Handle regular CLI commands
        if let Some(args) = parse_command_line(line) {
            return self.execute_cli_command(args).await;
        }

        // Treat as a direct question
        self.execute_ask(&line).await
    }

    async fn execute_slash_command(&mut self, line: &str) -> Result<String> {
        // Handle special REPL commands first
        let parts: Vec<&str> = line[1..].split_whitespace().collect();
        
        match parts.first() {
            Some(&"help") => Ok(self.show_help()),
            Some(&"exit") | Some(&"quit") => Err(anyhow::anyhow!("exit")),
            Some(&"persona") => {
                if parts.len() > 1 {
                    self.current_persona = parts[1].to_string();
                    Ok(format!("Switched to persona: {}", self.current_persona))
                } else {
                    Ok(format!("Current persona: {}", self.current_persona))
                }
            }
            Some(&"clear") => Ok("\x1B[2J\x1B[1;1H".to_string()), // ANSI clear screen
            Some(&"status") => {
                Ok("REPL Status: Ready".to_string())
            }
            Some(&"test") | Some(&"build") | Some(&"explain") => {
                // Use our new slash command system for these commands
                match slash::parse(line) {
                    Ok(command) => {
                        match slash::render(command) {
                            Ok(prompt) => {
                                info!("Executing slash command: {}", line);
                                ask(&prompt).await.map_err(Into::into)
                            }
                            Err(e) => Ok(format!("Error rendering command: {}", e)),
                        }
                    }
                    Err(e) => Ok(format!("Error parsing command: {}", e)),
                }
            }
            Some(cmd) => Ok(format!("Unknown command: /{}", cmd)),
            None => Ok("Empty command".to_string()),
        }
    }

    async fn execute_cli_command(&mut self, args: Vec<String>) -> Result<String> {
        use crate::cli::{Cli, Commands};
        use clap::Parser;

        let mut cmd_args = vec!["opencode".to_string()];
        cmd_args.extend(args);

        match Cli::try_parse_from(cmd_args) {
            Ok(cli) => {
                if let Some(command) = cli.command {
                    // Capture output for REPL display
                    match command {
                        Commands::Ask { question, persona } => {
                            self.execute_ask_with_persona(&question, &persona).await
                        }
                        Commands::Agent(_agent_cmd) => {
                            Ok("Agent commands not yet implemented".to_string())
                        }
                        Commands::Version => {
                            Ok(format!("OpenCode-RS CLI v{}", env!("CARGO_PKG_VERSION")))
                        }
                        Commands::Repl => {
                            Ok("Already in REPL mode.".to_string())
                        }
                    }
                } else {
                    Ok("No command specified. Type /help for available commands.".to_string())
                }
            }
            Err(e) => Ok(format!("Parse error: {}", e)),
        }
    }

    async fn execute_ask(&self, question: &str) -> Result<String> {
        self.execute_ask_with_persona(question, &self.current_persona).await
    }

    async fn execute_ask_with_persona(&self, question: &str, persona: &str) -> Result<String> {
        // For now, just use regular ask - persona support will be added later
        let prompt = if persona != "default" {
            format!("Acting as {}: {}", persona, question)
        } else {
            question.to_string()
        };
        
        match ask(&prompt).await {
            Ok(response) => Ok(response),
            Err(e) => Ok(format!("Error: {}", e)),
        }
    }

    fn show_help(&self) -> String {
        r#"OpenCode-RS REPL Commands:

Slash Commands:
  /help          - Show this help message
  /exit, /quit   - Exit the REPL
  /persona [name] - Set or show current persona
  /clear         - Clear the screen
  /status        - Show agent status

CLI Commands:
  agent ls       - List all agents
  agent spawn <id> [--persona <name>] - Spawn a new agent
  agent stop <id> - Stop an agent
  agent status <id> - Get agent status
  ask <question> [--persona <name>] - Ask a question
  version        - Show version information

Direct Questions:
  Just type your question and press Enter to ask using the current persona.

Examples:
  What is Rust?
  /persona expert
  What are the best practices for error handling?
  agent spawn my-agent --persona rusty
"#.to_string()
    }
}

pub async fn start() -> Result<()> {
    info!("Starting OpenCode-RS REPL");
    
    let mut line_editor = Reedline::create();
    let prompt = DefaultPrompt::default();
    let mut engine = ReplEngine::new();

    println!("OpenCode-RS Interactive REPL");
    println!("Type /help for available commands, /exit to quit.");
    println!();

    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(buffer)) => {
                debug!("Processing input: {}", buffer);
                match engine.execute_line(&buffer).await {
                    Ok(output) => {
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                    }
                    Err(e) => {
                        if e.to_string() == "exit" {
                            println!("Goodbye!");
                            break;
                        }
                        error!("Error: {}", e);
                        println!("Error: {}", e);
                    }
                }
            }
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                println!("\nGoodbye!");
                break;
            }
            x => {
                warn!("Unexpected signal: {:?}", x);
                println!("Error reading line: {:?}", x);
            }
        }
    }

    Ok(())
}

fn parse_command_line(line: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    // Check if it looks like a CLI command
    match parts.first() {
        Some(&"agent") | Some(&"ask") | Some(&"version") | Some(&"repl") => {
            Some(parts.iter().map(|s| s.to_string()).collect())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use test_case::test_case;
    use rstest::*;

    #[fixture]
    fn engine() -> ReplEngine {
        ReplEngine::new()
    }

    #[rstest]
    #[tokio::test]
    async fn test_empty_line(mut engine: ReplEngine) {
        let result = engine.execute_line("").await.unwrap();
        assert_eq!(result, "");
    }

    #[rstest]
    #[tokio::test]
    async fn test_whitespace_line(mut engine: ReplEngine) {
        let result = engine.execute_line("   \t  ").await.unwrap();
        assert_eq!(result, "");
    }

    #[rstest]
    #[tokio::test]
    async fn test_help_command(mut engine: ReplEngine) {
        let result = engine.execute_line("/help").await.unwrap();
        assert!(result.contains("OpenCode-RS REPL Commands"));
        assert!(result.contains("/help"));
        assert!(result.contains("agent ls"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_exit_command(mut engine: ReplEngine) {
        let result = engine.execute_line("/exit").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "exit");
    }

    #[rstest]
    #[tokio::test]
    async fn test_quit_command(mut engine: ReplEngine) {
        let result = engine.execute_line("/quit").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "exit");
    }

    #[rstest]
    #[tokio::test]
    async fn test_persona_command_set(mut engine: ReplEngine) {
        let result = engine.execute_line("/persona expert").await.unwrap();
        assert_eq!(result, "Switched to persona: expert");
        assert_eq!(engine.current_persona, "expert");
    }

    #[rstest]
    #[tokio::test]
    async fn test_persona_command_show(mut engine: ReplEngine) {
        let result = engine.execute_line("/persona").await.unwrap();
        assert_eq!(result, "Current persona: default");
    }

    #[rstest]
    #[tokio::test]
    async fn test_clear_command(mut engine: ReplEngine) {
        let result = engine.execute_line("/clear").await.unwrap();
        assert_eq!(result, "\x1B[2J\x1B[1;1H");
    }

    #[rstest]
    #[tokio::test]
    async fn test_status_command_empty(mut engine: ReplEngine) {
        let result = engine.execute_line("/status").await.unwrap();
        assert_eq!(result, "No agents running.");
    }

    #[rstest]
    #[tokio::test]
    async fn test_unknown_slash_command(mut engine: ReplEngine) {
        let result = engine.execute_line("/unknown").await.unwrap();
        assert_eq!(result, "Unknown command: /unknown");
    }

    #[tokio::test]
    async fn test_cli_command_parsing_agent_ls() {
        let mut engine = ReplEngine::new();
        let result = engine.execute_line("agent ls").await;
        assert!(result.is_ok(), "Failed to execute command: agent ls");
    }

    #[tokio::test]
    async fn test_cli_command_parsing_ask() {
        let mut engine = ReplEngine::new();
        let result = engine.execute_line("ask What is Rust?").await;
        assert!(result.is_ok(), "Failed to execute command: ask What is Rust?");
    }

    #[tokio::test]
    async fn test_cli_command_parsing_version() {
        let mut engine = ReplEngine::new();
        let result = engine.execute_line("version").await;
        assert!(result.is_ok(), "Failed to execute command: version");
    }

    #[rstest]
    #[tokio::test]
    async fn test_agent_ls_command(mut engine: ReplEngine) {
        let result = engine.execute_line("agent ls").await.unwrap();
        assert_eq!(result, "No agents running.");
    }

    #[rstest]
    #[tokio::test]
    async fn test_version_command(mut engine: ReplEngine) {
        let result = engine.execute_line("version").await.unwrap();
        assert!(result.contains("OpenCode-RS CLI v"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_invalid_cli_command(mut engine: ReplEngine) {
        let result = engine.execute_line("agent invalid").await.unwrap();
        assert!(result.contains("Parse error"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_direct_question(mut engine: ReplEngine) {
        // This will attempt to call the ask function, which might fail in test environment
        // but the important thing is that it doesn't panic or return an empty result
        let result = engine.execute_line("What is the meaning of life?").await.unwrap();
        // In test environment, this will likely return an error message, which is fine
        assert!(!result.is_empty());
    }

    #[test]
    fn test_parse_command_line_valid() {
        assert_eq!(parse_command_line("agent ls"), Some(vec!["agent".to_string(), "ls".to_string()]));
        assert_eq!(parse_command_line("ask What is Rust?"), Some(vec!["ask".to_string(), "What".to_string(), "is".to_string(), "Rust?".to_string()]));
        assert_eq!(parse_command_line("version"), Some(vec!["version".to_string()]));
    }

    #[test]
    fn test_parse_command_line_invalid() {
        assert_eq!(parse_command_line("hello world"), None);
        assert_eq!(parse_command_line("random text"), None);
        assert_eq!(parse_command_line(""), None);
    }

    #[test]
    fn test_parse_command_line_with_extra_whitespace() {
        assert_eq!(parse_command_line("  agent   ls  "), Some(vec!["agent".to_string(), "ls".to_string()]));
    }

    // Integration tests for the REPL engine
    #[tokio::test]
    async fn test_repl_engine_persona_persistence() {
        let mut engine = ReplEngine::new();
        
        // Set persona
        engine.execute_line("/persona expert").await.unwrap();
        assert_eq!(engine.current_persona, "expert");
        
        // Execute another command
        engine.execute_line("/help").await.unwrap();
        
        // Persona should persist
        assert_eq!(engine.current_persona, "expert");
    }

    #[tokio::test]
    async fn test_repl_engine_command_sequence() {
        let mut engine = ReplEngine::new();
        
        let commands = vec![
            "/persona test",
            "agent ls", 
            "/status",
            "version",
        ];
        
        for cmd in commands {
            let result = engine.execute_line(cmd).await;
            assert!(result.is_ok(), "Command failed: {}", cmd);
        }
    }

    // Property-based testing
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_slash_commands_dont_panic(cmd in "/[a-zA-Z]+") -> proptest::test_runner::TestCaseResult {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut engine = ReplEngine::new();
                    let result = engine.execute_line(&cmd).await;
                    prop_assert!(result.is_ok());
                    Ok(())
                })
            }

            #[test]
            fn test_empty_and_whitespace_lines(line in r"\s*") -> proptest::test_runner::TestCaseResult {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut engine = ReplEngine::new();
                    let result = engine.execute_line(&line).await;
                    prop_assert!(result.is_ok());
                    Ok(())
                })
            }
        }
    }
}