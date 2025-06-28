import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import App from '../App';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Get properly typed mocks
const mockInvoke = vi.mocked(invoke);
const mockListen = vi.mocked(listen);

describe('App Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockResolvedValue([]);
    mockListen.mockResolvedValue(() => {});
  });

  it('renders the main app structure', () => {
    render(<App />);
    
    expect(screen.getByText('OpenCode-RS')).toBeInTheDocument();
    expect(screen.getByText('Agents')).toBeInTheDocument();
    expect(screen.getByText('Spawn New Agent')).toBeInTheDocument();
    expect(screen.getByText('Swarm Monitor')).toBeInTheDocument();
    expect(screen.getByPlaceholderText('Type a message or slash command...')).toBeInTheDocument();
  });

  it('displays "No agents running" when no agents are present', async () => {
    mockInvoke.mockResolvedValue([]);
    
    render(<App />);
    
    await waitFor(() => {
      expect(screen.getByText('No agents running.')).toBeInTheDocument();
    });
  });

  it('displays agents when they exist', async () => {
    const mockAgents = [
      {
        id: 'test-agent',
        persona: 'rusty',
        status: 'Running',
        branch_name: 'agent-test-agent',
      },
    ];
    
    mockInvoke.mockResolvedValue(mockAgents);
    
    render(<App />);
    
    await waitFor(() => {
      expect(screen.getByText('test-agent')).toBeInTheDocument();
      expect(screen.getByText('Persona: rusty')).toBeInTheDocument();
      expect(screen.getByText('Status: 🟢 Running')).toBeInTheDocument();
    });
  });

  it('handles agent spawning', async () => {
    mockInvoke.mockResolvedValueOnce([]).mockResolvedValueOnce(undefined).mockResolvedValueOnce([
      {
        id: 'new-agent',
        persona: 'rusty',
        status: 'Running',
        branch_name: 'agent-new-agent',
      },
    ]);
    
    render(<App />);
    
    const idInput = screen.getByLabelText('Agent ID');
    const personaInput = screen.getByLabelText('Persona');
    const spawnButton = screen.getByText('Spawn');
    
    fireEvent.change(idInput, { target: { value: 'new-agent' } });
    fireEvent.change(personaInput, { target: { value: 'rusty' } });
    fireEvent.click(spawnButton);
    
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('spawn_agent', {
        id: 'new-agent',
        persona: 'rusty',
      });
    });
  });

  it('shows validation error for empty agent ID', async () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});
    mockInvoke.mockResolvedValue([]);
    
    render(<App />);
    
    const spawnButton = screen.getByText('Spawn');
    fireEvent.click(spawnButton);
    
    await waitFor(() => {
      expect(alertSpy).toHaveBeenCalledWith('Please provide both an agent ID and persona.');
    });
    
    alertSpy.mockRestore();
  });

  it('handles swarm build start', async () => {
    mockInvoke.mockResolvedValueOnce([]).mockResolvedValueOnce(undefined);
    
    render(<App />);
    
    const swarmButton = screen.getByText('Start Swarm Build');
    fireEvent.click(swarmButton);
    
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('execute_swarm_build');
      expect(screen.getByText('Running...')).toBeInTheDocument();
    });
  });

  it('handles swarm progress events', async () => {
    let eventCallback: any;
    mockListen.mockImplementation((event: string, callback: any) => {
      if (event === 'SWARM_PROGRESS') {
        eventCallback = callback;
      }
      return Promise.resolve(() => {});
    });
    
    render(<App />);
    
    // Wait for the SwarmMonitor to be rendered
    await waitFor(() => {
      expect(screen.getByText('Swarm Monitor')).toBeInTheDocument();
    });
    
    // Wait for component to mount and listener to be set up
    await waitFor(() => {
      expect(eventCallback).toBeDefined();
    });
    
    // Simulate progress event
    await act(async () => {
      eventCallback({
        payload: {
          total: 3,
          completed: 1,
          task: 'Building crate 1',
        },
      });
    });
    
    await waitFor(() => {
      expect(screen.getByText('1 / 3 tasks completed')).toBeInTheDocument();
      expect(screen.getByText('Building crate 1')).toBeInTheDocument();
    });
  });

  it('resets running state when swarm completes', async () => {
    let eventCallback: any;
    mockListen.mockImplementation((event: string, callback: any) => {
      if (event === 'SWARM_PROGRESS') {
        eventCallback = callback;
      }
      return Promise.resolve(() => {});
    });
    mockInvoke.mockResolvedValueOnce([]).mockResolvedValueOnce(undefined);
    
    render(<App />);
    
    // Wait for component to mount and listener to be set up
    await waitFor(() => {
      expect(eventCallback).toBeDefined();
    });
    
    // Start swarm build
    const swarmButton = screen.getByText('Start Swarm Build');
    fireEvent.click(swarmButton);
    
    await waitFor(() => {
      expect(screen.getByText('Running...')).toBeInTheDocument();
    });
    
    // Simulate completion event
    await act(async () => {
      eventCallback({
        payload: {
          total: 3,
          completed: 3,
          task: 'Swarm build finished!',
        },
      });
    });
    
    await waitFor(() => {
      expect(screen.getByText('Start Swarm Build')).toBeInTheDocument();
      expect(screen.queryByText('Running...')).not.toBeInTheDocument();
    });
  });
});