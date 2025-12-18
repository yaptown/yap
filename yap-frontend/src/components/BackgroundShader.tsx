import { useEffect, useRef, useMemo, memo, createContext, useContext, useCallback, ReactNode } from "react";
import { useTheme } from "./theme-provider";

interface BackgroundContextType {
  bumpBackground: (multiplier?: number) => void;
}

const BackgroundContext = createContext<BackgroundContextType | null>(null);

export function useBackground() {
  const context = useContext(BackgroundContext);
  if (!context) {
    throw new Error("useBackground must be used within a BackgroundShader");
  }
  return context;
}

interface BackgroundShaderProps {
  children: ReactNode;
}

function BackgroundShaderComponent({ children }: BackgroundShaderProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const workerRef = useRef<Worker | null>(null);
  const { theme, animatedBackground } = useTheme();

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

  // Check accessibility preferences and hardware capabilities
  const shouldRender = useMemo(() => {
    // User preference
    if (!animatedBackground) {
      return false;
    }

    // Disable on low-end devices
    if (navigator.hardwareConcurrency && navigator.hardwareConcurrency <= 4) {
      return false;
    }

    // Respect reduced motion preference
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      return false;
    }

    // Respect high contrast preference
    if (window.matchMedia("(prefers-contrast: more)").matches) {
      return false;
    }

    // Respect reduced transparency preference
    if (window.matchMedia("(prefers-reduced-transparency: reduce)").matches) {
      return false;
    }

    return true;
  }, [animatedBackground]);

  // Expose bump function to children
  const bumpBackground = useCallback((multiplier?: number) => {
    if (workerRef.current) {
      workerRef.current.postMessage({ type: 'bump', multiplier });
    }
  }, []);

  // Set up worker and transfer canvas control
  useEffect(() => {
    const container = containerRef.current;
    if (!container || !shouldRender) return;

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
      if (container.contains(canvas)) {
        container.removeChild(canvas);
      }
    };
  }, [shouldRender]); // Only re-run when shouldRender changes

  // Handle theme changes separately without recreating worker
  useEffect(() => {
    if (workerRef.current && shouldRender) {
      workerRef.current.postMessage({ type: 'theme', theme: actualTheme });
    }
  }, [actualTheme, shouldRender]);

  return (
    <BackgroundContext.Provider value={{ bumpBackground }}>
      {shouldRender && (
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
      )}
      {children}
    </BackgroundContext.Provider>
  );
}

export const BackgroundShader = memo(BackgroundShaderComponent);
