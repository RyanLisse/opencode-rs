# Slice 1.5: Core Refactoring - TDD Approach

## Overview

This document outlines the Test-Driven Development (TDD) approach for the core refactoring work that needs to be done before proceeding with Slice 2. The tests have been created FIRST to define the behavior we want, following the red-green-refactor cycle.

## Test Suite Structure

### 1. Provider Abstraction Tests (`opencode-core/src/provider/tests.rs`)

**Purpose**: Define the contract for AI provider implementations

**Key Test Cases**:
- ✅ Provider trait requirements and compilation checks
- ✅ Synchronous completion requests
- ✅ Streaming completion support
- ✅ Error handling and propagation
- ✅ Provider capabilities discovery
- ✅ Provider configuration

**Behaviors Defined**:
- All providers must implement async `complete` and `complete_stream` methods
- Providers must report their capabilities (streaming, function calling, vision, etc.)
- Errors must be properly categorized (API errors, rate limits, auth failures)
- Mock implementations available for testing

### 2. Configuration Management Tests (`opencode-core/src/config/tests.rs`)

**Purpose**: Define flexible configuration loading and management

**Key Test Cases**:
- ✅ Default configuration with sensible values
- ✅ Loading configuration from TOML files
- ✅ Environment variable expansion (${VAR} syntax)
- ✅ Configuration validation
- ✅ Configuration merging (base + overrides)
- ✅ Provider-specific configurations
- ✅ Hot reload support for configuration changes

**Behaviors Defined**:
- Configuration supports multiple providers with different settings
- Environment variables can be used for sensitive data
- Invalid configurations are caught early with clear errors
- Configuration can be updated without restarting

### 3. Dependency Injection Tests (`opencode-core/src/di/tests.rs`)

**Purpose**: Define a flexible DI container for managing dependencies

**Key Test Cases**:
- ✅ Container creation and service registration
- ✅ Singleton service lifetime
- ✅ Factory (transient) service lifetime
- ✅ Interface-based registration with named implementations
- ✅ Dependencies between services
- ✅ Scoped services for request-specific data
- ✅ Async service initialization
- ✅ Service not found error handling
- ✅ Builder pattern for container setup

**Behaviors Defined**:
- Services can have different lifetimes (singleton, transient, scoped)
- Dependencies are automatically resolved
- Interfaces can have multiple named implementations
- Async initialization is supported
- Clear errors when services are not registered

### 4. Error Handling Tests (`opencode-core/src/error/tests.rs`)

**Purpose**: Define comprehensive error handling with context and recovery

**Key Test Cases**:
- ✅ Error creation with proper display formatting
- ✅ Error context chaining
- ✅ Recovery suggestions for different error types
- ✅ Error source chain traversal
- ✅ Error categorization (transient, configuration, internal)
- ✅ Retry policies based on error type
- ✅ Error telemetry for debugging
- ✅ Error serialization to JSON
- ✅ Error aggregation for multiple failures
- ✅ Async error handling
- ✅ Error conversion from external libraries

**Behaviors Defined**:
- Errors carry context about where they occurred
- Transient errors have automatic retry policies
- Errors include recovery suggestions
- Full error chains are preserved for debugging
- Errors can be serialized for logging/API responses

## Implementation Approach

### Phase 1: Run All Tests (Red Phase)
1. Create the module structure in `opencode-core/src/lib.rs`
2. Create minimal type definitions to make tests compile
3. Run tests - all should fail (red)

### Phase 2: Implement Core Types (Green Phase)
1. Implement the Provider trait and basic types
2. Implement configuration structs and loading
3. Implement DI container with basic functionality
4. Implement error types with enhancement methods
5. Run tests - aim for all passing (green)

### Phase 3: Refactor and Optimize
1. Improve code organization
2. Add documentation
3. Optimize performance where needed
4. Ensure all tests still pass

## Benefits of This Approach

1. **Clear Requirements**: Tests define exactly what behavior we need
2. **Safety Net**: Tests ensure refactoring doesn't break functionality
3. **Documentation**: Tests serve as living documentation
4. **Design First**: TDD forces us to think about the API before implementation
5. **Confidence**: Comprehensive tests give confidence in the refactoring

## Next Steps

To implement the code that satisfies these tests:

1. Switch to the worktree: Work in `/Users/cortex/CascadeProjects/opencode-rs-slice-1.5`
2. Create the module structure
3. Implement each component to make its tests pass
4. Integrate the components together
5. Update existing code to use the new abstractions

## Test Execution

To run the tests:

```bash
# Run all tests
cargo test --package opencode-core

# Run specific test suite
cargo test --package opencode-core provider::tests
cargo test --package opencode-core config::tests
cargo test --package opencode-core di::tests
cargo test --package opencode-core error::tests

# Run with output
cargo test --package opencode-core -- --nocapture
```

## Integration Points

These core components will be used throughout the application:

- **Provider Abstraction**: Used by all AI provider implementations (OpenAI, Anthropic, etc.)
- **Configuration**: Loaded at startup and used to configure providers and server
- **Dependency Injection**: Wires together all application components
- **Error Handling**: Used consistently throughout for better debugging and recovery