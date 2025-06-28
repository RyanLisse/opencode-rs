// src/types.ts

export interface Agent {
  id: string;
  persona: string;
  status: 'Running' | 'Stopped' | { Error: string };
  branch_name: string;
}