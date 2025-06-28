import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { SwarmMonitor } from '../components/SwarmMonitor';

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

describe('SwarmMonitor Component', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(() => {});
  });

  it('renders the swarm monitor with start button', () => {
    render(<SwarmMonitor />);
    
    expect(screen.getByText('Swarm Monitor')).toBeInTheDocument();
    expect(screen.getByText('Start Swarm Build')).toBeInTheDocument();
  });

  it('calls execute_swarm_build when start button is clicked', async () => {
    mockInvoke.mockResolvedValue(undefined);
    
    render(<SwarmMonitor />);
    
    const startButton = screen.getByText('Start Swarm Build');
    fireEvent.click(startButton);
    
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith('execute_swarm_build');
    });
  });

  it('disables button and shows "Running..." when swarm is active', async () => {
    mockInvoke.mockResolvedValue(undefined);
    
    render(<SwarmMonitor />);
    
    const startButton = screen.getByText('Start Swarm Build');
    fireEvent.click(startButton);
    
    await waitFor(() => {
      expect(screen.getByText('Running...')).toBeInTheDocument();
      expect(screen.getByRole('button')).toBeDisabled();
    });
  });

  it('displays progress when progress events are received', async () => {
    let eventCallback: any;
    mockListen.mockImplementation((event: string, callback: any) => {
      if (event === 'SWARM_PROGRESS') {
        eventCallback = callback;
      }
      return Promise.resolve(() => {});
    });
    
    render(<SwarmMonitor />);
    
    // Simulate progress event
    if (eventCallback) {
      eventCallback({
        payload: {
          total: 5,
          completed: 2,
          task: 'Building core module',
        },
      });
    }
    
    await waitFor(() => {
      expect(screen.getByText('2 / 5 tasks completed')).toBeInTheDocument();
      expect(screen.getByText('Building core module')).toBeInTheDocument();
    });
  });

  it('resets to initial state when swarm completes', async () => {
    let eventCallback: any;
    mockListen.mockImplementation((event: string, callback: any) => {
      if (event === 'SWARM_PROGRESS') {
        eventCallback = callback;
      }
      return Promise.resolve(() => {});
    });
    
    render(<SwarmMonitor />);
    
    // Start swarm
    const startButton = screen.getByText('Start Swarm Build');
    fireEvent.click(startButton);
    
    await waitFor(() => {
      expect(screen.getByText('Running...')).toBeInTheDocument();
    });
    
    // Simulate completion
    if (eventCallback) {
      eventCallback({
        payload: {
          total: 3,
          completed: 3,
          task: 'Swarm build finished!',
        },
      });
    }
    
    await waitFor(() => {
      expect(screen.getByText('Start Swarm Build')).toBeInTheDocument();
      expect(screen.getByRole('button')).not.toBeDisabled();
    });
  });

  it('handles swarm build errors gracefully', async () => {
    const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});
    mockInvoke.mockRejectedValue(new Error('Build failed'));
    
    render(<SwarmMonitor />);
    
    const startButton = screen.getByText('Start Swarm Build');
    fireEvent.click(startButton);
    
    await waitFor(() => {
      expect(alertSpy).toHaveBeenCalledWith('Error: Error: Build failed');
      expect(screen.getByText('Start Swarm Build')).toBeInTheDocument();
      expect(screen.getByRole('button')).not.toBeDisabled();
    });
    
    alertSpy.mockRestore();
  });

  it('calculates progress percentage correctly', async () => {
    let eventCallback: any;
    mockListen.mockImplementation((event: string, callback: any) => {
      if (event === 'SWARM_PROGRESS') {
        eventCallback = callback;
      }
      return Promise.resolve(() => {});
    });
    
    render(<SwarmMonitor />);
    
    // Simulate 75% progress
    if (eventCallback) {
      eventCallback({
        payload: {
          total: 4,
          completed: 3,
          task: 'Almost done...',
        },
      });
    }
    
    await waitFor(() => {
      expect(screen.getByText('3 / 4 tasks completed')).toBeInTheDocument();
      expect(screen.getByText('Almost done...')).toBeInTheDocument();
    });
  });

  it('handles zero progress correctly', async () => {
    let eventCallback: any;
    mockListen.mockImplementation((event: string, callback: any) => {
      if (event === 'SWARM_PROGRESS') {
        eventCallback = callback;
      }
      return Promise.resolve(() => {});
    });
    
    render(<SwarmMonitor />);
    
    // Simulate starting state
    if (eventCallback) {
      eventCallback({
        payload: {
          total: 5,
          completed: 0,
          task: 'Starting swarm build...',
        },
      });
    }
    
    await waitFor(() => {
      expect(screen.getByText('0 / 5 tasks completed')).toBeInTheDocument();
      expect(screen.getByText('Starting swarm build...')).toBeInTheDocument();
    });
  });
});