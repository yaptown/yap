import { useEffect, useRef, useMemo, memo } from "react";
import { useTheme } from "./theme-provider";

function BackgroundShaderComponent() {
  const containerRef = useRef<HTMLDivElement>(null);
  const workerRef = useRef<Worker | null>(null);
  const { theme } = useTheme();

  // Determine actual theme (resolve "system") - memoized to prevent recalculation
  const actualTheme = useMemo(
    () =>
      theme === "system"
        ? window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light"
        : theme,
    [theme]
  );

  // Set up worker and transfer canvas control
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // Create a fresh canvas element for this worker
    const canvas = document.createElement("canvas");
    canvas.className = "fixed inset-0 w-full h-full -z-10";
    canvas.style.pointerEvents = "none";
    canvas.style.willChange = "contents";
    canvas.style.transform = "translateZ(0)";
    container.appendChild(canvas);

    // Create worker
    const worker = new Worker(
      new URL("../workers/backgroundShader.worker.ts", import.meta.url),
      { type: "module" }
    );
    workerRef.current = worker;

    // Transfer canvas control to worker
    const offscreenCanvas = canvas.transferControlToOffscreen();

    // Set initial size
    const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
    const scale = 0.75;
    offscreenCanvas.width = window.innerWidth * dpr * scale;
    offscreenCanvas.height = window.innerHeight * dpr * scale;

    // Initialize worker with canvas and theme
    worker.postMessage(
      {
        type: "init",
        canvas: offscreenCanvas,
        theme: actualTheme,
      },
      [offscreenCanvas]
    );

    // Handle resize events
    const handleResize = () => {
      worker.postMessage({
        type: "resize",
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    window.addEventListener("resize", handleResize);

    // Cleanup
    return () => {
      window.removeEventListener("resize", handleResize);
      worker.postMessage({ type: "stop" });
      worker.terminate();
      workerRef.current = null;
      container.removeChild(canvas);
    };
  }, [actualTheme]); // Re-run when theme changes

  return (
    <>
      <div ref={containerRef} className="contents" />
      <div
        className="fixed inset-0 w-full h-full -z-10 opacity-[0.30]"
        style={{
          pointerEvents: "none",
          backgroundImage: "url(/noise2.webp)",
          backgroundRepeat: "no-repeat",
          backgroundSize: "cover",
          backgroundPosition: "center",
          mixBlendMode: actualTheme === "dark" ? "multiply" : "screen",
          filter: actualTheme === "dark" ? "invert(1)" : "none",
        }}
      />
    </>
  );
}

export const BackgroundShader = memo(BackgroundShaderComponent);
