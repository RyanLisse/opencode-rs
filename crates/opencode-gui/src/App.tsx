import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Agent } from "./types";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { SwarmMonitor } from './components/SwarmMonitor';

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
      <SwarmMonitor />
      <Card>
        <CardHeader>
          <CardTitle>Agents</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-2">
          {(!agents || agents.length === 0) && <p className="text-muted-foreground">No agents running.</p>}
          {agents && agents.map((agent) => (
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