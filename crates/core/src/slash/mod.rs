use crate::personas::{load_personas, Persona};
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::fs;

#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub struct Command {
    pub name: String,
    pub persona: Option<Persona>,
    pub file_path: Option<String>,
}

/// Parses a user input line that starts with `/`.
pub fn parse(line: &str) -> Result<Command> {
    let personas = load_personas()?;
    parse_with_personas(line, personas)
}

/// Parses a slash command with custom personas (for testing)
pub fn parse_with_personas(line: &str, personas: HashMap<String, Persona>) -> Result<Command> {
    let mut cmd = Command::default();
    let args: Vec<&str> = line.split_whitespace().collect();
    
    if args.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    // First argument is the command name
    cmd.name = args[0].trim_start_matches('/').to_string();
    
    // Parse remaining arguments manually
    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--persona" | "-p" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Missing persona name after --persona"));
                }
                let persona_name = args[i + 1];
                cmd.persona = Some(
                    personas
                        .get(persona_name)
                        .cloned()
                        .context(format!("Persona '{}' not found", persona_name))?
                );
                i += 2;
            }
            "--file" | "-f" => {
                if i + 1 >= args.len() {
                    return Err(anyhow!("Missing file path after --file"));
                }
                cmd.file_path = Some(args[i + 1].to_string());
                i += 2;
            }
            arg if arg.starts_with("--") => {
                return Err(anyhow!("Unknown flag: {}", arg));
            }
            arg if arg.starts_with("-") && arg.len() > 1 => {
                return Err(anyhow!("Unknown short flag: {}", arg));
            }
            _ => {
                return Err(anyhow!("Unexpected argument: {}", args[i]));
            }
        }
    }

    Ok(cmd)
}

/// Renders a parsed command into a final prompt for the AI.
pub fn render(cmd: Command) -> Result<String> {
    let mut final_prompt = String::new();

    // 1. Add the persona's system prompt if it exists.
    if let Some(persona) = &cmd.persona {
        final_prompt.push_str(&format!(
            "SYSTEM PROMPT: {}\n\n---\n\n",
            persona.system_prompt
        ));
    }

    // 2. Add context from a file if provided.
    if let Some(path) = &cmd.file_path {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;
        final_prompt.push_str(&format!(
            "CONTEXT FROM FILE ({}):\n```\n{}\n```\n\n---\n\n",
            path, content
        ));
    }

    // 3. Add the main task based on the command name.
    let task = match cmd.name.as_str() {
        "test" => "Based on the context from the file, please write a comprehensive suite of unit tests for the code. Cover edge cases.",
        "build" => "Based on the context from the file, analyze the code for potential build issues or improvements.",
        "explain" => "Explain the code provided in the context file. Describe its purpose, how it works, and any potential improvements.",
        _ => return Err(anyhow!("Unknown slash command: /{}", cmd.name)),
    };
    final_prompt.push_str(&format!("TASK: {}\n", task));

    Ok(final_prompt)
}