// The app's animated background: a WebGL shader run by a worker on an
// OffscreenCanvas, so the main thread never waits on it. `mountShader` puts
// one into any positioned element; the app uses it fixed behind every page
// and the landing uses it inside one section.
import { getBackgroundPalette, type ShaderTheme } from "./shader-colors";
import { mountWorkerCanvas } from "./worker-canvas";

let webgl2Available: boolean | undefined;
function supportsWebGL2(): boolean {
  if (webgl2Available === undefined) {
    try {
      const gl = new OffscreenCanvas(1, 1).getContext("webgl2");
      webgl2Available = gl !== null;
      gl?.getExtension("WEBGL_lose_context")?.loseContext();
    } catch {
      webgl2Available = false;
    }
  }
  return webgl2Available;
}

/** Whether the shader should run at all: the user's setting, then the
 *  device's and the platform's say. */
export function shaderAvailable(animatedBackground: boolean): boolean {
  if (!animatedBackground) return false;
  if (navigator.hardwareConcurrency && navigator.hardwareConcurrency < 4)
    return false;
  for (const q of [
    "(prefers-reduced-motion: reduce)",
    "(prefers-contrast: more)",
    "(prefers-reduced-transparency: reduce)",
  ]) {
    if (window.matchMedia(q).matches) return false;
  }
  return (
    typeof HTMLCanvasElement.prototype.transferControlToOffscreen ===
      "function" && supportsWebGL2()
  );
}

export interface ShaderHandle {
  setTheme(theme: ShaderTheme): void;
  /** Speed the drift up for a moment (a card flip, a rating). */
  bump(multiplier?: number): void;
  /** Move the "sun" anchor, in canvas fractions with y upward. */
  setMouse(x: number, y: number): void;
  stop(): void;
}

const DEFAULT_MOUSE: [number, number] = [0.5, 0.4];

/** Fill `container` (which must be positioned) with the shader. The worker
 *  follows the container's size, and the sun follows a mouse over it while
 *  `mouseFollow()` says so and the container is on screen. */
export function mountShader(
  container: HTMLElement,
  theme: ShaderTheme,
  mouseFollow: () => boolean,
): ShaderHandle {
  const worker = new Worker(
    new URL("../workers/backgroundShader.worker.ts", import.meta.url),
    { type: "module" },
  );
  const mounted = mountWorkerCanvas(container, worker, {
    theme,
    palette: getBackgroundPalette(theme),
  });
  // Reveal only after the worker has compiled and drawn successfully. Context
  // loss and compile errors must expose the themed CSS fallback, not black.
  const canvas = container.querySelector("canvas")!;
  canvas.style.visibility = "hidden";
  worker.addEventListener("message", ({ data }) => {
    if (data.type === "ready") canvas.style.visibility = "visible";
    if (data.type === "unavailable") canvas.style.visibility = "hidden";
  });
  worker.addEventListener("error", () => {
    canvas.style.visibility = "hidden";
  });

  const setMouse = (x: number, y: number) =>
    worker.postMessage({ type: "mouse", x, y });
  // y is flipped to match the shader's bottom-up UV. Touch devices never
  // fire this and the sun stays at its anchor.
  const onPointerMove = (e: PointerEvent) => {
    if (e.pointerType !== "mouse" || !mouseFollow() || !mounted.onScreen())
      return;
    const r = container.getBoundingClientRect();
    setMouse(
      Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)),
      Math.min(1, Math.max(0, 1 - (e.clientY - r.top) / r.height)),
    );
  };
  const onPointerLeave = () => setMouse(...DEFAULT_MOUSE);
  window.addEventListener("pointermove", onPointerMove);
  document.documentElement.addEventListener("pointerleave", onPointerLeave);

  return {
    setTheme: (t) =>
      worker.postMessage({
        type: "theme",
        theme: t,
        palette: getBackgroundPalette(t),
      }),
    bump: (multiplier) => worker.postMessage({ type: "bump", multiplier }),
    setMouse,
    stop() {
      window.removeEventListener("pointermove", onPointerMove);
      document.documentElement.removeEventListener(
        "pointerleave",
        onPointerLeave,
      );
      mounted.stop();
    },
  };
}
