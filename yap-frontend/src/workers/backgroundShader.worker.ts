// Simple test worker for BackgroundShader
// This demonstrates basic worker setup and communication

interface WorkerMessage {
  type: string;
  payload?: Record<string, unknown>;
}

let frameCount = 0;
let startTime = performance.now();

// Listen for messages from the main thread
self.addEventListener('message', (event: MessageEvent<WorkerMessage>) => {
  const { type, payload } = event.data;

  switch (type) {
    case 'ping': {
      // Respond with pong and some stats
      const elapsed = performance.now() - startTime;
      self.postMessage({
        type: 'pong',
        payload: {
          frameCount,
          elapsed,
          timestamp: performance.now(),
        },
      });
      frameCount++;
      break;
    }

    case 'calculate': {
      // Example of doing some work in the worker
      const { a, b } = payload as { a: number; b: number };
      const result = a + b;
      self.postMessage({
        type: 'result',
        payload: { result },
      });
      break;
    }

    case 'reset': {
      // Reset the worker state
      frameCount = 0;
      startTime = performance.now();
      self.postMessage({
        type: 'reset-complete',
      });
      break;
    }

    default: {
      console.warn('[Worker] Unknown message type:', type);
      break;
    }
  }
});

// Send a ready message
self.postMessage({
  type: 'ready',
  payload: {
    message: 'Worker is ready!',
  },
});
