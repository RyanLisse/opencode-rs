import '@testing-library/jest-dom';

// Mock Tauri APIs globally
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Mock window.alert
Object.defineProperty(window, 'alert', {
  writable: true,
  value: vi.fn(),
});