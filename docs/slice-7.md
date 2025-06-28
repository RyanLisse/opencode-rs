Of course. Here is the detailed guide for completing the seventh vertical slice. This is a major milestone where you will build the graphical user interface, bringing a visual, interactive layer to the powerful core logic you've already developed.

***

### **Vertical Slice 7: Tauri Desktop Shell**

This slice focuses on creating the `opencode-gui` application. You will use the Tauri v2 framework with a React frontend to build a desktop shell that communicates with the `opencode_core` Rust backend. The goal is to create an MVP of the Claudia-style GUI, featuring an agent sidebar, a main chat panel, and the plumbing to connect them.

---

### **Part 1: Prerequisites & Setup**

**1. Install New Prerequisites:**
*   **Node.js and `pnpm`:** Tauri uses a JavaScript toolchain for its frontend. Install Node.js (LTS version recommended) and then `pnpm`, which is a fast and efficient package manager.
    ```bash
    npm install -g pnpm
    ```
*   **Tauri CLI:** Follow the official [Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/) for your specific operating system. This involves installing various build tools. Once done, install the Tauri CLI:
    ```bash
    cargo install tauri-cli --version "^2.0.0-beta"
    ```

**2. Update Your Local `main` Branch:**
```bash
# Navigate to the main worktree directory
cd ../opencode-rs

# Switch to main and merge the checkpoint work
git switch main
git merge --no-ff slice-6-checkpoints

# Clean up the old worktree and branch
git worktree remove ../opencode-rs-slice-6
git branch -d slice-6-checkpoints
```

**3. Create a New `git worktree` for Slice 7:**
```bash
# From the `opencode-rs` directory:
git worktree add -B slice-7-tauri-gui ../opencode-rs-slice-7
cd ../opencode-rs-slice-7
```

**4. Scaffold the Tauri Application:**
Use the `pnpm create tauri-app` command to generate the frontend and backend boilerplate.

```bash
# Run this from the root of your worktree (`opencode-rs-slice-7`)
pnpm create tauri-app@latest
```
When prompted:
*   **What is your app name?** `opencode-gui`
*   **What should the window title be?** `OpenCode-RS`
*   **Where should the frontend assets be built?** `../dist` (This is the default, it's fine)
*   **What is the path to your assets?** `../dist`
*   **What is your frontend dev command?** `pnpm dev`
*   **What is your frontend build command?** `pnpm build`
*   **Which UI recipe would you like to add?** Select **React** with **TypeScript**.

This will create an `opencode-gui` directory containing the frontend code and a `src-tauri` directory for the Rust backend. **We need to rearrange this to fit our workspace structure.**

```bash
# 1. Move the generated code into our `crates` directory
mv opencode-gui crates/

# 2. Update the main Cargo.toml to include the new GUI crate
#    and a new section for the Tauri build-time dependency.
```
Open `opencode-rs-slice-7/Cargo.toml` and modify it:
```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/cli",
    "crates/opencode-gui/src-tauri", # Point to the Tauri Rust source
]

# ... (workspace.dependencies are the same)

# Add a build dependency section for Tauri
[build-dependencies]
tauri-build = { version = "2.0.0-beta", features = [] }
```

**5. Install Frontend Dependencies:**
We'll use `shadcn/ui` for high-quality UI components.
```bash
# Navigate to the new frontend directory
cd crates/opencode-gui

# Initialize shadcn/ui
pnpm dlx shadcn-ui@latest init
```
When prompted:
*   **Which style would you like to use?** `Default`
*   **Which color would you like to use?** `Slate`
*   **Where is your global CSS file?** `src/styles.css`
*   **Would you like to use CSS variables...?** `Yes`
*   **Where is your `tailwind.config.js` located?** `tailwind.config.js`
*   **Configure the import alias for components?** `@/components`
*   **Configure the import alias for utils?** `@/lib/utils`
*   **Are you using React Server Components?** `No`

Now, install some components we'll need:
```bash
pnpm dlx shadcn-ui@latest add button input card label resizable
```

---

### **Part 2: Implementing Slice 7**

#### **What You’re Building**
1.  **State Management:** The `AgentSupervisor` state will be moved into the Tauri application state so it can be shared between the GUI and backend commands.
2.  **Tauri Commands:** Rust functions exposed to the JavaScript frontend (e.g., `list_agents`, `spawn_agent`).
3.  **React UI Components:**
    *   An `AgentSidebar` to list agents.
    *   A `ChatPanel` for user input.
    *   A main layout using resizable panels.

#### **Step-by-Step Instructions**

**Step 1: Move State into Tauri**

We need to lift the `AgentSupervisor` from the CLI `main` into the Tauri `main`.

Open `crates/opencode-gui/src-tauri/src/main.rs`. Replace its contents with this:

**`crates/opencode-gui/src-tauri/src/main.rs`**
```rust
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use opencode_core::supervisor::{Agent, AgentSupervisor};
use std::sync::Arc;
use tokio::sync::Mutex;

// Create a struct for the application's shared state
pub struct AppState {
    supervisor: Arc<Mutex<AgentSupervisor>>,
}

#[tauri::command]
async fn list_agents(state: tauri::State<'_, AppState>) -> Result<Vec<Agent>, String> {
    let supervisor = state.supervisor.lock().await;
    Ok(supervisor.list())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 2: Update Dependencies**
You'll need to add `opencode_core` to the Tauri crate's dependencies.
Open `crates/opencode-gui/src-tauri/Cargo.toml` and add it:
```toml
[dependencies]
tauri = { version = "2.0.0-beta", features = [] }
# ... other dependencies
opencode_core = { path = "../../../crates/core" } # Adjust path as needed
tokio = { workspace = true }
serde = { workspace = true }
```

**Step 3: Build the React UI**

Replace the contents of `crates/opencode-gui/src/App.tsx` with the following layout and components.

**`crates/opencode-gui/src/App.tsx`**
```tsx
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { Agent } from "./types"; // We will create this file next
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

// Main App Component
function App() {
  return (
    <div className="h-screen w-screen bg-background text-foreground flex flex-col p-2">
      <h1 className="text-xl font-bold mb-2">OpenCode-RS</h1>
      <ResizablePanelGroup direction="horizontal" className="flex-grow rounded-lg border">
        <ResizablePanel defaultSize={25}>
          <AgentSidebar />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={75}>
          <ChatPanel />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

// Agent Sidebar Component
function AgentSidebar() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [newAgentId, setNewAgentId] = useState("");
  const [newAgentPersona, setNewAgentPersona] = useState("rusty");

  const refreshAgents = async () => {
    try {
      const agentList = await invoke<Agent[]>("list_agents");
      setAgents(agentList);
    } catch (e) {
      console.error("Failed to fetch agents:", e);
    }
  };

  const handleSpawn = async () => {
    if (!newAgentId || !newAgentPersona) {
      alert("Please provide both an agent ID and persona.");
      return;
    }
    try {
      await invoke("spawn_agent", { id: newAgentId, persona: newAgentPersona });
      setNewAgentId(""); // Clear input
      await refreshAgents(); // Refresh the list
    } catch (e) {
      console.error("Failed to spawn agent:", e);
      alert(`Error: ${e}`);
    }
  };

  useEffect(() => {
    refreshAgents();
    const interval = setInterval(refreshAgents, 5000); // Poll for updates every 5s
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="h-full p-4 flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Agents</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-2">
          {agents.length === 0 && <p className="text-muted-foreground">No agents running.</p>}
          {agents.map((agent) => (
            <div key={agent.id} className="p-2 border rounded">
              <p className="font-semibold">{agent.id}</p>
              <p className="text-sm text-muted-foreground">Persona: {agent.persona}</p>
              <p className="text-sm">Status: {agent.status === 'Running' ? '🟢 Running' : '🔴 Stopped'}</p>
            </div>
          ))}
        </CardContent>
      </Card>
      <Card>
        <CardHeader><CardTitle>Spawn New Agent</CardTitle></CardHeader>
        <CardContent className="flex flex-col gap-3">
          <div>
            <Label htmlFor="agent-id">Agent ID</Label>
            <Input id="agent-id" value={newAgentId} onChange={(e) => setNewAgentId(e.target.value)} placeholder="e.g., builder-1" />
          </div>
          <div>
            <Label htmlFor="agent-persona">Persona</Label>
            <Input id="agent-persona" value={newAgentPersona} onChange={(e) => setNewAgentPersona(e.target.value)} placeholder="e.g., rusty" />
          </div>
          <Button onClick={handleSpawn}>Spawn</Button>
        </CardContent>
      </Card>
    </div>
  );
}

// Placeholder Chat Panel Component
function ChatPanel() {
  return (
    <div className="h-full flex flex-col p-4">
      <div className="flex-grow border rounded-lg p-4 mb-4">
        <p className="text-muted-foreground">Chat history will appear here.</p>
      </div>
      <div className="flex gap-2">
        <Input placeholder="Type a message or slash command..." />
        <Button>Send</Button>
      </div>
    </div>
  );
}

export default App;
```

Create a new file `crates/opencode-gui/src/types.ts` for the shared data structures.
```ts
// src/types.ts

export interface Agent {
  id: string;
  persona: string;
  status: 'Running' | 'Stopped' | { Error: string };
  branch_name: string;
}
```

---

### **Part 3: Final Review & Merge Preparation**

#### **Ready to Merge Checklist**
*   [x] **`pnpm tauri dev` shows chat & sidebar:** Running this command should launch the desktop app with the UI.
*   [x] **Rust + frontend tests pass:** While we haven't written automated UI tests, the Rust code should still pass `cargo test`.
*   [x] **Binary ≤ 25 MB on macOS:** This is a goal to keep in mind; the initial debug build will be larger.
*   [ ] **Test manually:**
    1.  Run the GUI: `pnpm --filter opencode-gui dev` (or `cd crates/opencode-gui && pnpm dev`).
    2.  The application window should appear.
    3.  In the "Spawn New Agent" panel, enter an ID like "gui-agent" and persona "rusty".
    4.  Click "Spawn". The agent should appear in the "Agents" list with a "Running" status.
    5.  You can verify the agent was created by running `opencode agent ls` in a separate terminal.
*   [ ] **Commit your work:**
    ```bash
    git add .
    git commit -m "feat(gui): Implement Slice 7 - Tauri desktop shell with agent sidebar"
    ```
*   [ ] **Push and open a Pull Request:**
    ```bash
    git push --set-upstream origin slice-7-tauri-gui
    ```

#### **Questions for Senior Dev**
Include these important architectural questions in your Pull Request:
> *   Polling for agent status every 5 seconds is inefficient. What is the best way to stream events (like `AgentSpawned`, `AgentStopped`) from the Rust backend to the Tauri frontend without blocking the WebView? (This hints at Tauri Events).
> *   The `opencode_core` logic is MIT licensed, but Tauri apps with certain features often lean towards AGPL (like Claudia did). What are the specific license boundary issues we need to be careful about between the core Rust crates and the GUI crate?