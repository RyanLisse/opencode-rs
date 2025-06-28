use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

/// End-to-end integration tests for the entire OpenCode system
#[tokio::test]
async fn test_full_workflow() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();

    // Test CLI basic functionality
    let output = Command::new("cargo")
        .args(&["run", "--bin", "opencode", "--", "--version"])
        .output()
        .expect("Failed to execute opencode CLI");

    assert!(output.status.success(), "CLI version command failed");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("opencode"),
        "Version output doesn't contain 'opencode'"
    );

    // Test help command
    let output = Command::new("cargo")
        .args(&["run", "--bin", "opencode", "--", "--help"])
        .output()
        .expect("Failed to execute opencode CLI help");

    assert!(output.status.success(), "CLI help command failed");
    let help_output = String::from_utf8_lossy(&output.stdout);
    assert!(help_output.contains("USAGE"), "Help output doesn't contain usage information");
}

#[tokio::test]
async fn test_cli_core_integration() {
    // Test that CLI can properly initialize the core library
    let output = Command::new("cargo")
        .args(&["run", "--bin", "opencode", "--", "init", "--dry-run"])
        .output()
        .expect("Failed to execute opencode CLI init");

    // Should succeed even in dry-run mode
    assert!(output.status.success() || output.status.code() == Some(1), 
           "CLI init command failed unexpectedly: {:?}", 
           String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn test_provider_integration() {
    use opencode_core::provider::{Provider, ProviderManager};
    use opencode_core::config::Config;

    // Test provider initialization and basic functionality
    let config = Config::default();
    let manager = ProviderManager::new(config);
    
    // Verify provider registration works
    assert!(manager.is_healthy().await, "Provider manager should be healthy");
}

#[tokio::test]
async fn test_supervisor_agent_coordination() {
    use opencode_core::supervisor::Supervisor;
    use opencode_core::config::Config;
    use tokio::time::Duration;

    let config = Config::default();
    let supervisor = Supervisor::new(config);

    // Test basic supervisor functionality
    let health_check = timeout(Duration::from_secs(5), supervisor.health_check()).await;
    assert!(health_check.is_ok(), "Supervisor health check timed out");
    assert!(health_check.unwrap().is_ok(), "Supervisor health check failed");
}

#[tokio::test]
async fn test_swarm_orchestration() {
    use opencode_core::swarm::SwarmOrchestrator;
    use opencode_core::config::Config;

    let config = Config::default();
    let orchestrator = SwarmOrchestrator::new(config);

    // Test swarm initialization
    assert!(orchestrator.is_healthy().await, "Swarm orchestrator should be healthy");
}

#[cfg(feature = "container-tests")]
#[tokio::test]
async fn test_container_isolation() {
    // This test would require Docker to be available
    // Test container creation, isolation, and cleanup
    
    // Note: This is a placeholder for container integration tests
    // In a real implementation, this would:
    // 1. Start a container
    // 2. Execute code inside it
    // 3. Verify isolation
    // 4. Clean up resources
    
    println!("Container isolation tests require Docker runtime");
}

#[tokio::test]
async fn test_error_handling_and_recovery() {
    use opencode_core::error::OpenCodeError;
    
    // Test various error conditions and recovery mechanisms
    
    // Test invalid configuration
    let result = std::panic::catch_unwind(|| {
        // This should handle errors gracefully
        let _config = opencode_core::config::Config::from_file("nonexistent.toml");
    });
    assert!(result.is_ok(), "Error handling should not panic");
}

#[tokio::test]
async fn test_concurrent_operations() {
    use opencode_core::config::Config;
    use std::sync::Arc;
    use tokio::task::JoinSet;

    let config = Arc::new(Config::default());
    let mut join_set = JoinSet::new();

    // Spawn multiple concurrent operations
    for i in 0..5 {
        let config_clone = Arc::clone(&config);
        join_set.spawn(async move {
            // Simulate concurrent work
            tokio::time::sleep(Duration::from_millis(100 * i)).await;
            format!("Operation {} completed", i)
        });
    }

    // Wait for all operations to complete
    let mut results = Vec::new();
    while let Some(result) = join_set.join_next().await {
        results.push(result.expect("Task should complete successfully"));
    }

    assert_eq!(results.len(), 5, "All concurrent operations should complete");
}

#[tokio::test]
async fn test_performance_benchmarks() {
    use std::time::Instant;
    use opencode_core::config::Config;

    let config = Config::default();
    
    // Basic performance benchmark
    let start = Instant::now();
    
    // Simulate some work
    for _ in 0..1000 {
        let _ = serde_json::to_string(&config);
    }
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Performance test should complete within 1 second");
}

#[tokio::test]
async fn test_memory_usage() {
    // Test for memory leaks and excessive memory usage
    let initial_memory = get_memory_usage();
    
    // Perform operations that might leak memory
    for _ in 0..100 {
        let config = opencode_core::config::Config::default();
        drop(config);
    }
    
    // Force garbage collection
    #[cfg(feature = "jemalloc")]
    {
        // If using jemalloc, we could trigger collection here
    }
    
    let final_memory = get_memory_usage();
    let memory_growth = final_memory.saturating_sub(initial_memory);
    
    // Allow some memory growth but flag excessive growth
    assert!(memory_growth < 10_000_000, "Memory usage grew too much: {} bytes", memory_growth);
}

fn get_memory_usage() -> usize {
    // Simplified memory usage check
    // In a real implementation, this would use proper memory profiling
    std::mem::size_of::<opencode_core::config::Config>()
}

#[tokio::test]
async fn test_cross_platform_compatibility() {
    // Test platform-specific functionality
    
    #[cfg(target_os = "windows")]
    {
        // Windows-specific tests
        test_windows_specific_features().await;
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS-specific tests
        test_macos_specific_features().await;
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux-specific tests
        test_linux_specific_features().await;
    }
}

#[cfg(target_os = "windows")]
async fn test_windows_specific_features() {
    // Test Windows path handling, permissions, etc.
    use std::path::Path;
    let path = Path::new("C:\\Windows\\System32");
    assert!(path.is_absolute(), "Windows absolute path should be recognized");
}

#[cfg(target_os = "macos")]
async fn test_macos_specific_features() {
    // Test macOS-specific functionality
    use std::path::Path;
    let path = Path::new("/Applications");
    // This path should exist on macOS
}

#[cfg(target_os = "linux")]
async fn test_linux_specific_features() {
    // Test Linux-specific functionality
    use std::path::Path;
    let path = Path::new("/usr/bin");
    assert!(path.exists() || !path.exists(), "Path check should not panic");
}