# Slice 3: Slash Commands with Personas Implementation Plan

## Overview
Building on Slice 2's CLI foundation, Slice 3 will implement a comprehensive persona system with slash command parsing for context injection and command rendering.

## Implementation Approach

### 1. Directory Structure
```
crates/personas/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── loader.rs      # YAML persona loading
│   ├── parser.rs      # Slash command parsing with lexopt
│   ├── renderer.rs    # Command rendering and context injection
│   └── types.rs       # Persona data structures
└── tests/
    ├── fixtures/
    │   └── test_personas.yml
    └── integration_tests.rs
```

### 2. Core Components

#### A. Persona System (`types.rs`)
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub commands: Vec<SlashCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub template: String,
    pub context_vars: Vec<String>,
}
```

#### B. Configuration Loading (`loader.rs`)
```rust
pub struct PersonaLoader {
    config_dir: PathBuf,
}

impl PersonaLoader {
    pub fn from_config_dir() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Unable to find config directory"))?
            .join("opencode");
        Ok(Self { config_dir })
    }

    pub fn load_personas(&self) -> Result<HashMap<String, Persona>> {
        // Load all .yml files from ~/.config/opencode/personas/
    }
}
```

#### C. Command Parsing (`parser.rs`)
```rust
pub struct SlashCommandParser {
    personas: HashMap<String, Persona>,
}

impl SlashCommandParser {
    pub fn parse(&self, input: &str) -> Result<ParsedCommand> {
        // Use lexopt for robust command line parsing
        // Handle: /persona_name:command_name --flag value args...
    }
}

#[derive(Debug)]
pub struct ParsedCommand {
    pub persona: String,
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, String>,
}
```

#### D. Template Rendering (`renderer.rs`)
```rust
pub struct CommandRenderer {
    personas: HashMap<String, Persona>,
}

impl CommandRenderer {
    pub fn render(&self, parsed: ParsedCommand, context: &Context) -> Result<RenderedCommand> {
        // Template variable substitution
        // Context injection (git status, file paths, etc.)
    }
}

#[derive(Debug)]
pub struct RenderedCommand {
    pub persona: Persona,
    pub rendered_prompt: String,
    pub context: String,
}
```

### 3. Test-Driven Development Plan

#### Phase 1: Persona Loading Tests
```rust
#[test]
fn test_load_personas_from_yaml() {
    let loader = PersonaLoader::from_test_dir();
    let personas = loader.load_personas().unwrap();
    assert!(personas.contains_key("architect"));
    assert!(personas.contains_key("rusty"));
}

#[test]
fn test_persona_validation() {
    // Test required fields, valid templates, etc.
}

#[test]
fn test_config_directory_creation() {
    // Test that ~/.config/opencode/ is created if missing
}
```

#### Phase 2: Command Parsing Tests
```rust
#[test_case("/architect:design --api rest UserService"; "basic command")]
#[test_case("/rusty:review src/main.rs --strict"; "file path argument")]
#[test_case("/qa:test --coverage 90"; "flag with value")]
fn test_slash_command_parsing(input: &str) {
    let parser = SlashCommandParser::new(test_personas());
    let result = parser.parse(input);
    assert!(result.is_ok());
}

#[test]
fn test_unknown_persona_error() {
    let parser = SlashCommandParser::new(test_personas());
    let result = parser.parse("/unknown:command");
    assert!(result.is_err());
}

#[test]
fn test_invalid_syntax_error() {
    let parser = SlashCommandParser::new(test_personas());
    let result = parser.parse("not_a_slash_command");
    assert!(result.is_err());
}
```

#### Phase 3: Template Rendering Tests
```rust
#[test]
fn test_template_variable_substitution() {
    let renderer = CommandRenderer::new(test_personas());
    let context = Context::new()
        .with_var("file_path", "src/main.rs")
        .with_var("language", "rust");
    
    let parsed = ParsedCommand { /* ... */ };
    let rendered = renderer.render(parsed, &context).unwrap();
    
    assert!(rendered.rendered_prompt.contains("src/main.rs"));
    assert!(rendered.rendered_prompt.contains("rust"));
}

#[test]
fn test_context_injection() {
    // Test git status, file content injection, etc.
}
```

#### Phase 4: Integration Tests
```rust
#[tokio::test]
async fn test_end_to_end_slash_command() {
    let cli = setup_test_cli();
    let result = cli.execute("/architect:design --api rest UserService").await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("architecture"));
}
```

### 4. Example Persona Configuration

#### `~/.config/opencode/personas/architect.yml`
```yaml
name: "architect"
description: "System architecture and design expert"
system_prompt: |
  You are a senior software architect with 15+ years of experience.
  You excel at system design, scalability, and best practices.
  Always consider maintainability, performance, and team productivity.
temperature: 0.3
max_tokens: 2000

commands:
  - name: "design"
    description: "Create system architecture for a component"
    template: |
      Design a {{style}} architecture for {{component}}.
      
      Requirements:
      {{#if api_type}}
      - API Type: {{api_type}}
      {{/if}}
      {{#if scalability}}
      - Scalability: {{scalability}}
      {{/if}}
      
      Current project context:
      {{git_status}}
      
      Focus on:
      - Maintainability
      - Scalability
      - Team productivity
      - Security considerations
    context_vars:
      - "git_status"
      - "project_structure"

  - name: "review"
    description: "Architectural review of code or design"
    template: |
      Perform an architectural review of:
      {{file_content}}
      
      Focus areas:
      - Design patterns usage
      - SOLID principles adherence
      - Scalability considerations
      - Security implications
    context_vars:
      - "file_content"
```

#### `~/.config/opencode/personas/rusty.yml`
```yaml
name: "rusty"
description: "Rust programming expert"
system_prompt: |
  You are a Rust expert who loves systems programming, memory safety,
  and performance optimization. You know the ecosystem deeply.
temperature: 0.4
max_tokens: 1500

commands:
  - name: "review"
    description: "Review Rust code for best practices"
    template: |
      Review this Rust code for best practices:
      
      ```rust
      {{file_content}}
      ```
      
      Check for:
      - Memory safety patterns
      - Performance optimizations
      - Idiomatic Rust usage
      - Error handling
      {{#if strict}}
      - Apply strict clippy-level scrutiny
      {{/if}}
    context_vars:
      - "file_content"

  - name: "optimize"
    description: "Suggest performance optimizations"
    template: |
      Analyze this Rust code for performance optimization opportunities:
      
      ```rust
      {{file_content}}
      ```
      
      Consider:
      - Memory allocation patterns
      - Iterator usage
      - Async/await optimization
      - SIMD opportunities
    context_vars:
      - "file_content"
```

### 5. CLI Integration

#### Enhanced REPL Support
```rust
// In repl.rs
async fn execute_slash_command(&mut self, line: &str) -> Result<String> {
    if let Some(parsed) = self.slash_parser.parse(line)? {
        let context = self.build_context().await?;
        let rendered = self.renderer.render(parsed, &context)?;
        
        // Execute with persona-specific settings
        let response = opencode_core::ask_with_persona(
            &rendered.rendered_prompt,
            &rendered.persona.name
        ).await?;
        
        Ok(response)
    } else {
        // Fall back to existing slash command handling
        self.execute_builtin_slash_command(line).await
    }
}
```

### 6. Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum PersonaError {
    #[error("Persona '{0}' not found")]
    PersonaNotFound(String),
    
    #[error("Command '{0}' not found for persona '{1}'")]
    CommandNotFound(String, String),
    
    #[error("Invalid slash command syntax: {0}")]
    InvalidSyntax(String),
    
    #[error("Template rendering failed: {0}")]
    TemplateError(String),
    
    #[error("Context variable '{0}' not available")]
    MissingContextVar(String),
}
```

### 7. Performance Considerations

- **Lazy Loading**: Personas loaded once and cached
- **Template Compilation**: Pre-compile templates for reuse
- **Context Caching**: Cache expensive context operations
- **Async Operations**: Non-blocking file and git operations

### 8. Test Coverage Goals

- **Persona Loading**: 100% - Critical for system functionality
- **Command Parsing**: 95% - Handle edge cases and malformed input
- **Template Rendering**: 90% - Cover common patterns and errors
- **Integration**: 85% - End-to-end scenarios

### 9. Development Phases

1. **Phase 1** (2-3 hours): Basic persona loading and data structures
2. **Phase 2** (2-3 hours): Slash command parsing with lexopt
3. **Phase 3** (2-3 hours): Template rendering and context injection
4. **Phase 4** (1-2 hours): CLI integration and REPL enhancement
5. **Phase 5** (1 hour): Documentation and examples

### 10. Success Criteria

- [ ] Load personas from `~/.config/opencode/personas/*.yml`
- [ ] Parse slash commands: `/persona:command --flags args`
- [ ] Render templates with context variable substitution
- [ ] Integrate with existing CLI and REPL
- [ ] Achieve >90% test coverage
- [ ] Handle all error conditions gracefully
- [ ] Provide comprehensive usage examples

This implementation will provide a powerful, extensible persona system that enhances the CLI with context-aware command processing while maintaining the clean architecture established in Slice 2.