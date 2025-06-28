#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use opencode_core::supervisor::{Agent, AgentSupervisor};
use opencode_core::swarm;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

// Create a struct for the application's shared state
pub struct AppState {
    supervisor: Arc<Mutex<AgentSupervisor>>,
}

// Define the payload for our progress event
#[derive(Clone, serde::Serialize)]
struct SwarmProgressPayload {
    total: usize,
    completed: usize,
    task: String,
}

#[tauri::command]
async fn list_agents(state: tauri::State<'_, AppState>) -> Result<Vec<Agent>, String> {
    let supervisor = state.supervisor.lock().await;
    Ok(supervisor.list().await)
}

#[tauri::command]
async fn spawn_agent(
    id: String,
    persona: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut supervisor = state.supervisor.lock().await;
    supervisor
        .spawn(&id, &persona)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn execute_swarm_build(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let supervisor = state.supervisor.lock().await;

    // For this example, we assume Cargo.toml is in the current directory.
    let manifest_path = PathBuf::from("Cargo.toml");
    let plan = swarm::plan_build_from_manifest(&manifest_path).map_err(|e| e.to_string())?;

    let total_tasks = plan.tasks.len();
    println!("Executing swarm build with {} tasks.", total_tasks);

    // Emit initial event
    app_handle.emit("SWARM_PROGRESS", SwarmProgressPayload {
        total: total_tasks,
        completed: 0,
        task: "Starting swarm build...".into(),
    }).unwrap();

    // Drop the supervisor lock before spawning tasks
    drop(supervisor);

    // Spawn an agent for each task
    for (i, task) in plan.tasks.iter().enumerate() {
        let agent_id = format!("builder-{}", task.replace('/', "-"));
        let persona = "rusty"; // Use a default builder persona
        
        // Acquire lock for each spawn operation
        let mut supervisor = state.supervisor.lock().await;
        supervisor.spawn(&agent_id, persona).await.map_err(|e| e.to_string())?;
        drop(supervisor);

        // Simulate work being done
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Emit a progress event after each task
        app_handle.emit("SWARM_PROGRESS", SwarmProgressPayload {
            total: total_tasks,
            completed: i + 1,
            task: format!("Completed build for '{}'", task),
        }).unwrap();
    }
    
    // Final completion event
    app_handle.emit("SWARM_PROGRESS", SwarmProgressPayload {
        total: total_tasks,
        completed: total_tasks,
        task: "Swarm build finished!".into(),
    }).unwrap();

    Ok(())
}

fn main() {
    // Create the initial state
    let state = AppState {
        supervisor: Arc::new(Mutex::new(AgentSupervisor::new())),
    };

    tauri::Builder::default()
        .manage(state) // Add the state to be managed by Tauri
        .invoke_handler(tauri::generate_handler![
            // Register our commands
            list_agents,
            spawn_agent,
            execute_swarm_build,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}