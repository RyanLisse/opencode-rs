# SuperClaude Configuration

This directory contains project-specific configuration for [SuperClaude](https://github.com/NomenAK/SuperClaude), enhancing the AI assistant's understanding of the OpenCode-RS project.

## What is SuperClaude?

SuperClaude is a configuration system that optimizes Claude's behavior for software development by providing:
- Project-specific context and guidelines
- Structured commands and personas
- Workflow automation
- Token efficiency

## Files in this Directory

- **CLAUDE.md**: Main project configuration file containing:
  - Project overview and architecture
  - Development workflow and conventions
  - Current state and roadmap
  - Commands and environment setup

## How to Use

When working with Claude (Desktop, VSCode, or API), the assistant will automatically reference these files to:
- Understand project structure and conventions
- Follow established workflows
- Make informed architectural decisions
- Generate code that fits the project style

## Benefits

1. **Consistency**: All team members get the same AI assistance
2. **Context**: Claude understands the project without repeated explanations
3. **Efficiency**: Faster, more accurate responses
4. **Onboarding**: New developers can quickly understand the project

## Updating Configuration

When making significant changes to the project:
1. Update `.claude/CLAUDE.md` with new information
2. Commit changes with clear messages
3. All team members will benefit from updated context

## Example Usage

```bash
# Use personas for different tasks
/persona:architect "Design the CLI command structure"
/persona:backend "Implement the REPL using Clap"

# Use thinking modes for complex tasks
/user:build --think "Add error handling to the ask function"

# Reference project conventions
"Follow the project's async patterns for the new feature"
```

## Learn More

- [SuperClaude Repository](https://github.com/NomenAK/SuperClaude)
- [Project Documentation](../README.md)