//! Integration tests for OpenCode-RS
//! 
//! This module contains comprehensive integration tests that verify
//! the interaction between different components of the system.

pub mod end_to_end;
pub mod gui_backend_ipc;
pub mod multi_agent_coordination;
pub mod container_isolation;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init_test_env() {
    INIT.call_once(|| {
        // Set up logging for tests
        env_logger::init();
        
        // Set test-specific environment variables
        std::env::set_var("RUST_LOG", "debug");
        std::env::set_var("OPENCODE_TEST_MODE", "true");
    });
}

/// Common test utilities
pub mod utils {
    use std::path::PathBuf;
    use tempfile::TempDir;
    
    pub fn create_test_workspace() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let workspace_path = temp_dir.path().to_path_buf();
        (temp_dir, workspace_path)
    }
    
    pub fn is_ci_environment() -> bool {
        std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
    }
    
    pub async fn wait_for_service(port: u16, timeout_secs: u64) -> bool {
        use tokio::time::{timeout, Duration};
        use tokio::net::TcpStream;
        
        let result = timeout(
            Duration::from_secs(timeout_secs),
            async {
                loop {
                    if TcpStream::connect(format!("127.0.0.1:{}", port)).await.is_ok() {
                        return true;
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        ).await;
        
        result.unwrap_or(false)
    }
}