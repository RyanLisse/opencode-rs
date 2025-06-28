import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { Progress } from "@/components/ui/progress";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from './ui/button';

interface SwarmProgress {
  total: number;
  completed: number;
  task: string;
}

export function SwarmMonitor() {
  const [progress, setProgress] = useState<SwarmProgress | null>(null);
  const [isRunning, setIsRunning] = useState(false);

  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen<SwarmProgress>('SWARM_PROGRESS', (event) => {
        setProgress(event.payload);
        if (event.payload.completed === event.payload.total) {
          setIsRunning(false);
        }
      });

      return unlisten;
    };

    setupListener().then(unlisten => {
      return () => {
        unlisten();
      };
    });
  }, []);

  const handleStartSwarm = async () => {
    setIsRunning(true);
    setProgress(null); // Reset progress
    try {
      await invoke('execute_swarm_build');
    } catch (e) {
      console.error("Failed to execute swarm build:", e);
      alert(`Error: ${e}`);
      setIsRunning(false);
    }
  };

  const percentage = progress ? (progress.completed / progress.total) * 100 : 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle>Swarm Monitor</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <Button onClick={handleStartSwarm} disabled={isRunning}>
          {isRunning ? "Running..." : "Start Swarm Build"}
        </Button>
        {progress && (
          <div className="flex flex-col gap-2">
            <Progress value={percentage} />
            <p className="text-sm text-muted-foreground text-center">
              {progress.completed} / {progress.total} tasks completed
            </p>
            <p className="text-sm text-center">{progress.task}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}