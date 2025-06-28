use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[cfg(test)]
mod tests;

/// Represents a persona with a name and system prompt
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Persona {
    pub name: String,
    #[serde(rename = "system-prompt")]
    pub system_prompt: String,
}

/// Loads personas from the configuration file
pub fn load_personas() -> Result<HashMap<String, Persona>> {
    let config_path = get_config_path()?.join("personas.yml");
    if !config_path.exists() {
        return Ok(HashMap::new());
    }

    let file_content = fs::read_to_string(config_path)?;
    let personas: Vec<Persona> = serde_yml::from_str(&file_content)
        .context("Failed to parse personas.yml")?;

    let persona_map = personas
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    Ok(persona_map)
}

/// Loads personas from a specific file path (for testing)
pub fn load_personas_from_path(path: &PathBuf) -> Result<HashMap<String, Persona>> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let file_content = fs::read_to_string(path)?;
    let personas: Vec<Persona> = serde_yml::from_str(&file_content)
        .context("Failed to parse personas.yml")?;

    let persona_map = personas
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect();

    Ok(persona_map)
}

/// Gets the configuration directory path
fn get_config_path() -> Result<PathBuf> {
    let config_dir = directories::ProjectDirs::from("dev", "opencode", "opencode")
        .context("Could not determine config directory")?
        .config_dir()
        .to_path_buf();

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }
    Ok(config_dir)
}

/// Gets the configuration directory path without creating it
pub fn get_config_path_no_create() -> Result<PathBuf> {
    let config_dir = directories::ProjectDirs::from("dev", "opencode", "opencode")
        .context("Could not determine config directory")?
        .config_dir()
        .to_path_buf();
    Ok(config_dir)
}