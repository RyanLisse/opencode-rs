# OpenCode-RS Project Configuration

## Project Overview
OpenCode-RS is an AI-powered coding suite that combines:
- **Rust Core** (`opencode_core`): High-performance business logic
- **CLI Interface**: Terminal-based interaction (upcoming)
- **Tauri GUI**: Desktop application (future)
- **Agent Orchestration**: Containerized AI agents via Dagger

## Architecture Philosophy
- **Safety First**: Rust's memory safety and type system
- **Modular Design**: Separate crates for different concerns
- **Agent Isolation**: Each agent runs in its own container with dedicated git worktree
- **AI-Native**: Built from ground up for AI-assisted development

## Development Workflow
1. **Git Worktrees**: Each feature slice gets its own worktree
2. **Vertical Slices**: Complete features from core to UI
3. **TDD Approach**: Tests first, implementation second
4. **Continuous Integration**: Lint, test, and validate before merge

## Current State
- ✅ **Slice 1**: Core library with OpenAI integration
- 🔄 **PR #1**: Core skeleton implementation (awaiting review)
- 📋 **Next**: Slice 2 - CLI with REPL

## Commands & Automation
```bash
make build    # Build all crates
make test     # Run unit tests
make lint     # Check code quality with clippy
make clean    # Remove build artifacts
make help     # Show all available commands
```

## Environment Setup
```bash
# Required environment variables
OPENAI_API_KEY="sk-your-key-here"  # In .env file

# Rust toolchain
rustc 1.88.0+  # Minimum required version
```

## Project Structure
```
opencode-rs/
├── Cargo.toml          # Workspace definition
├── Makefile            # Build automation
├── .env                # Local environment (gitignored)
├── crates/
│   ├── core/           # Business logic
│   ├── cli/            # Terminal interface (upcoming)
│   └── gui/            # Tauri desktop app (future)
└── .claude/            # SuperClaude configuration
```

## Key Design Decisions
1. **Lazy Client Initialization**: API clients created once, reused
2. **Error Handling**: Using `anyhow` for ergonomic error management
3. **Async-First**: All I/O operations are async
4. **Workspace Structure**: Shared dependencies, separate concerns

## Testing Strategy
- **Unit Tests**: Fast, isolated, run frequently
- **Integration Tests**: API calls, marked with `#[ignore]`
- **E2E Tests**: Full workflow validation (future)

## AI Integration Points
1. **LLM Providers**: Currently OpenAI, extensible to others
2. **Agent Personas**: Different AI behaviors for different tasks
3. **Container Isolation**: Safe execution environment for agents
4. **Git Integration**: Version control for AI-generated code

## Performance Considerations
- **Token Efficiency**: Minimize API calls and token usage
- **Caching**: Reuse clients and responses where possible
- **Parallel Execution**: Agents can work concurrently
- **Resource Limits**: Container constraints for safety

## Security Principles
- **No Secrets in Code**: Use environment variables
- **Isolated Execution**: Containers prevent cross-contamination
- **Least Privilege**: Agents only access their worktree
- **Audit Trail**: All actions logged and versioned

## Future Roadmap
1. **CLI Implementation**: Interactive terminal interface
2. **Command System**: Slash commands and personas
3. **Git Worktree Management**: Automated branch handling
4. **Container Integration**: Dagger-based agent runtime
5. **GUI Development**: Tauri-based desktop application
6. **Swarm Orchestration**: Multi-agent collaboration

## Development Tips
- Use `cargo watch` for auto-rebuild during development
- Run `make lint` before committing
- Keep PRs focused on single slices
- Document architectural decisions
- Write tests for critical paths

## SuperClaude Integration
This project is optimized for development with SuperClaude:
- Use `/persona:architect` for design decisions
- Use `/persona:backend` for Rust implementation
- Use `--think` flag for complex features
- Reference this file for project context