use super::*;
use pretty_assertions::assert_eq;
use rstest::*;
use std::fs;
use tempfile::TempDir;
use test_case::test_case;

#[fixture]
fn temp_config_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

#[rstest]
fn test_persona_struct_creation() {
    let persona = Persona {
        name: "test".to_string(),
        system_prompt: "You are a test persona".to_string(),
    };
    
    assert_eq!(persona.name, "test");
    assert_eq!(persona.system_prompt, "You are a test persona");
}

#[rstest]
fn test_persona_serialization() {
    let persona = Persona {
        name: "rusty".to_string(),
        system_prompt: "You are a Rust expert".to_string(),
    };

    let serialized = serde_yml::to_string(&persona).expect("Failed to serialize");
    assert!(serialized.contains("name: rusty"));
    assert!(serialized.contains("system-prompt: You are a Rust expert"));
}

#[rstest]
fn test_persona_deserialization() {
    let yaml_content = r#"
name: "security-expert"
system-prompt: "You are a cybersecurity expert"
"#;

    let persona: Persona = serde_yml::from_str(yaml_content).expect("Failed to deserialize");
    assert_eq!(persona.name, "security-expert");
    assert_eq!(persona.system_prompt, "You are a cybersecurity expert");
}

#[rstest]
fn test_load_personas_from_nonexistent_file(temp_config_dir: TempDir) {
    let nonexistent_path = temp_config_dir.path().join("nonexistent.yml");
    let result = load_personas_from_path(&nonexistent_path).expect("Should handle missing file");
    assert!(result.is_empty());
}

#[rstest]
fn test_load_personas_from_empty_file(temp_config_dir: TempDir) {
    let personas_path = temp_config_dir.path().join("personas.yml");
    fs::write(&personas_path, "[]").expect("Failed to write file");

    let result = load_personas_from_path(&personas_path).expect("Should handle empty file");
    assert!(result.is_empty());
}

#[rstest]
fn test_load_personas_from_valid_file(temp_config_dir: TempDir) {
    let personas_path = temp_config_dir.path().join("personas.yml");
    let yaml_content = r#"
- name: "rusty"
  system-prompt: "You are a senior Rust developer"
- name: "security-expert"  
  system-prompt: "You are a cybersecurity expert"
"#;
    fs::write(&personas_path, yaml_content).expect("Failed to write file");

    let result = load_personas_from_path(&personas_path).expect("Should load personas");
    assert_eq!(result.len(), 2);
    
    let rusty = result.get("rusty").expect("Should contain rusty persona");
    assert_eq!(rusty.name, "rusty");
    assert_eq!(rusty.system_prompt, "You are a senior Rust developer");

    let security = result.get("security-expert").expect("Should contain security-expert persona");
    assert_eq!(security.name, "security-expert");
    assert_eq!(security.system_prompt, "You are a cybersecurity expert");
}

#[test_case("- name: invalid\n  bad-field: test" ; "invalid field")]
#[test_case("invalid yaml content" ; "invalid yaml")]
#[test_case("- name: missing-system-prompt" ; "missing required field")]
fn test_load_personas_invalid_yaml(yaml_content: &str) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let personas_path = temp_dir.path().join("personas.yml");
    fs::write(&personas_path, yaml_content).expect("Failed to write file");

    let result = load_personas_from_path(&personas_path);
    assert!(result.is_err());
}

#[rstest]
fn test_get_config_path_no_create() {
    let path = get_config_path_no_create().expect("Should get config path");
    assert!(path.to_string_lossy().contains("opencode"));
}

#[rstest]
fn test_duplicate_persona_names_overwrites() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let personas_path = temp_dir.path().join("personas.yml");
    let yaml_content = r#"
- name: "duplicate"
  system-prompt: "First prompt"
- name: "duplicate"
  system-prompt: "Second prompt"
"#;
    fs::write(&personas_path, yaml_content).expect("Failed to write file");

    let result = load_personas_from_path(&personas_path).expect("Should load personas");
    assert_eq!(result.len(), 1);
    
    let persona = result.get("duplicate").expect("Should contain persona");
    assert_eq!(persona.system_prompt, "Second prompt"); // Last one wins
}