use super::*;
use crate::personas::Persona;
use pretty_assertions::assert_eq;
use rstest::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use test_case::test_case;

#[fixture]
fn sample_personas() -> HashMap<String, Persona> {
    let mut personas = HashMap::new();
    personas.insert(
        "rusty".to_string(),
        Persona {
            name: "rusty".to_string(),
            system_prompt: "You are a senior Rust developer".to_string(),
        },
    );
    personas.insert(
        "security".to_string(),
        Persona {
            name: "security".to_string(),
            system_prompt: "You are a cybersecurity expert".to_string(),
        },
    );
    personas
}

#[fixture]
fn temp_file() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test.rs");
    fs::write(&test_file, "fn main() {\n    println!(\"Hello, world!\");\n}")
        .expect("Failed to write test file");
    temp_dir
}

#[rstest]
fn test_command_default() {
    let cmd = Command::default();
    assert_eq!(cmd.name, "");
    assert!(cmd.persona.is_none());
    assert!(cmd.file_path.is_none());
}

#[rstest]
fn test_parse_simple_command(sample_personas: HashMap<String, Persona>) {
    let result = parse_with_personas("/test", sample_personas).expect("Should parse command");
    assert_eq!(result.name, "test");
    assert!(result.persona.is_none());
    assert!(result.file_path.is_none());
}

#[rstest]
fn test_parse_command_with_persona(sample_personas: HashMap<String, Persona>) {
    let result = parse_with_personas("/test --persona rusty", sample_personas)
        .expect("Should parse command with persona");
    
    assert_eq!(result.name, "test");
    assert!(result.persona.is_some());
    assert_eq!(result.persona.unwrap().name, "rusty");
    assert!(result.file_path.is_none());
}

#[rstest]
fn test_parse_command_with_file(sample_personas: HashMap<String, Persona>) {
    let result = parse_with_personas("/build --file src/main.rs", sample_personas)
        .expect("Should parse command with file");
    
    assert_eq!(result.name, "build");
    assert!(result.persona.is_none());
    assert_eq!(result.file_path, Some("src/main.rs".to_string()));
}

#[rstest]
fn test_parse_command_with_both_flags(sample_personas: HashMap<String, Persona>) {
    let result = parse_with_personas("/explain --persona security --file test.rs", sample_personas)
        .expect("Should parse command with both flags");
    
    assert_eq!(result.name, "explain");
    assert!(result.persona.is_some());
    assert_eq!(result.persona.unwrap().name, "security");
    assert_eq!(result.file_path, Some("test.rs".to_string()));
}

#[rstest]
fn test_parse_command_short_flags(sample_personas: HashMap<String, Persona>) {
    let result = parse_with_personas("/test -p rusty -f main.rs", sample_personas)
        .expect("Should parse command with short flags");
    
    assert_eq!(result.name, "test");
    assert!(result.persona.is_some());
    assert_eq!(result.persona.unwrap().name, "rusty");
    assert_eq!(result.file_path, Some("main.rs".to_string()));
}

#[rstest]
fn test_parse_unknown_persona(sample_personas: HashMap<String, Persona>) {
    let result = parse_with_personas("/test --persona unknown", sample_personas);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test_case("/test --invalid-flag" ; "unknown flag")]
#[test_case("/test --persona" ; "missing persona value")]
#[test_case("/test --file" ; "missing file value")]
fn test_parse_invalid_commands(command: &str) {
    let personas = HashMap::new();
    let result = parse_with_personas(command, personas);
    assert!(result.is_err());
}

#[rstest]
fn test_render_simple_command() {
    let cmd = Command {
        name: "test".to_string(),
        persona: None,
        file_path: None,
    };
    
    let result = render(cmd).expect("Should render command");
    assert!(result.contains("TASK: Based on the context"));
    assert!(!result.contains("SYSTEM PROMPT"));
    assert!(!result.contains("CONTEXT FROM FILE"));
}

#[rstest]
fn test_render_command_with_persona() {
    let persona = Persona {
        name: "rusty".to_string(),
        system_prompt: "You are a Rust expert".to_string(),
    };
    
    let cmd = Command {
        name: "build".to_string(),
        persona: Some(persona),
        file_path: None,
    };
    
    let result = render(cmd).expect("Should render command with persona");
    assert!(result.contains("SYSTEM PROMPT: You are a Rust expert"));
    assert!(result.contains("TASK: Based on the context"));
    assert!(!result.contains("CONTEXT FROM FILE"));
}

#[rstest]
fn test_render_command_with_file(temp_file: TempDir) {
    let file_path = temp_file.path().join("test.rs");
    let cmd = Command {
        name: "explain".to_string(),
        persona: None,
        file_path: Some(file_path.to_string_lossy().to_string()),
    };
    
    let result = render(cmd).expect("Should render command with file");
    assert!(result.contains("CONTEXT FROM FILE"));
    assert!(result.contains("Hello, world!"));
    assert!(result.contains("TASK: Explain the code"));
    assert!(!result.contains("SYSTEM PROMPT"));
}

#[rstest]
fn test_render_command_with_both(temp_file: TempDir) {
    let persona = Persona {
        name: "security".to_string(),
        system_prompt: "You are a security expert".to_string(),
    };
    let file_path = temp_file.path().join("test.rs");
    
    let cmd = Command {
        name: "test".to_string(),
        persona: Some(persona),
        file_path: Some(file_path.to_string_lossy().to_string()),
    };
    
    let result = render(cmd).expect("Should render command with both");
    assert!(result.contains("SYSTEM PROMPT: You are a security expert"));
    assert!(result.contains("CONTEXT FROM FILE"));
    assert!(result.contains("Hello, world!"));
    assert!(result.contains("TASK: Based on the context"));
}

#[rstest]
fn test_render_unknown_command() {
    let cmd = Command {
        name: "unknown".to_string(),
        persona: None,
        file_path: None,
    };
    
    let result = render(cmd);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown slash command"));
}

#[rstest]
fn test_render_with_nonexistent_file() {
    let cmd = Command {
        name: "test".to_string(),
        persona: None,
        file_path: Some("/nonexistent/file.rs".to_string()),
    };
    
    let result = render(cmd);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to read file"));
}

#[test_case("test", "Based on the context from the file, please write a comprehensive suite of unit tests" ; "test command")]
#[test_case("build", "analyze the code for potential build issues or improvements" ; "build command")]  
#[test_case("explain", "Explain the code provided in the context file" ; "explain command")]
fn test_command_task_descriptions(command_name: &str, expected_task: &str) {
    let cmd = Command {
        name: command_name.to_string(),
        persona: None,
        file_path: None,
    };
    
    let result = render(cmd).expect("Should render command");
    assert!(result.contains(expected_task));
}