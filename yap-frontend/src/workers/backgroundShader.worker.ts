// Background shader worker - handles all canvas rendering off the main thread

import type { ShaderTheme } from "../lib/shader-colors";
import type { BackgroundPalette } from "yap-frontend-rs";
import fragmentShaderSrc from "./background.generated.frag?raw";

interface WorkerMessage {
  type: string;
  canvas?: OffscreenCanvas;
  theme?: ShaderTheme;
  palette?: BackgroundPalette;
  width?: number;
  height?: number;
  devicePixelRatio?: number;
  multiplier?: number;
  x?: number;
  y?: number;
}

let gl: WebGL2RenderingContext | null = null;
let canvas: OffscreenCanvas | null = null;
let currentTheme: ShaderTheme = "dark";
let animationFrameId: number | null = null;

// Overloaded function signatures
function zeno(
  current: number,
  target: number,
  delta_time: number,
  rate?: number,
): number;
function zeno(
  current: number[],
  target: number[],
  delta_time: number,
  rate?: number,
): number[];
function zeno(
  current: number[][],
  target: number[][],
  delta_time: number,
  rate?: number,
): number[][];

// Implementation
function zeno(
  current: number | number[] | number[][],
  target: number | number[] | number[][],
  delta_time: number,
  rate = 5.0,
): number | number[] | number[][] {
  const alpha = 1 - Math.exp(-rate * delta_time);

  // Scalar case
  if (typeof current === "number" && typeof target === "number") {
    return current + alpha * (target - current);
  }

  // Array case
  if (Array.isArray(current) && Array.isArray(target)) {
    // Check if it's array of arrays
    if (Array.isArray(current[0]) && Array.isArray(target[0])) {
      return (current as number[][]).map((row, i) =>
        row.map((val, j) => val + alpha * ((target as number[][])[i][j] - val)),
      );
    }

    // Array of numbers
    return (current as number[]).map(
      (val, i) => val + alpha * ((target as number[])[i] - val),
    );
  }

  throw new Error("Invalid types for zeno function");
}

function createShader(
  gl: WebGL2RenderingContext,
  type: number,
  source: string,
): WebGLShader | null {
  const shader = gl.createShader(type);
  if (!shader) return null;
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    console.error("Shader compile error:", gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }
  return shader;
}

function createProgram(
  gl: WebGL2RenderingContext,
  vertexShader: WebGLShader,
  fragmentShader: WebGLShader,
): WebGLProgram | null {
  const program = gl.createProgram();
  if (!program) return null;
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error("Program link error:", gl.getProgramInfoLog(program));
    return null;
  }
  return program;
}

/** Size the drawing buffer for a box: a capped pixel ratio and a fraction
 *  of the resolution, since the shader is all soft gradients. */
function fit(width: number, height: number, devicePixelRatio: number) {
  if (!canvas) return;
  const dpr = Math.min(devicePixelRatio, 1.5);
  const isMobile = width < 768;
  const scale = isMobile ? 0.35 : 0.75;
  canvas.width = width * dpr * scale;
  canvas.height = height * dpr * scale;
  gl?.viewport(0, 0, canvas.width, canvas.height);
}

function initWebGL(
  offscreenCanvas: OffscreenCanvas,
  theme: ShaderTheme,
  palette: BackgroundPalette,
  width: number,
  height: number,
  devicePixelRatio: number,
) {
  canvas = offscreenCanvas;
  currentTheme = theme;
  canvas.addEventListener("webglcontextlost", () => {
    if (animationFrameId !== null) cancelAnimationFrame(animationFrameId);
    self.postMessage({ type: "unavailable" });
  });
  fit(width, height, devicePixelRatio);

  gl = canvas.getContext("webgl2", {
    alpha: false,
    antialias: false,
    depth: false,
    stencil: false,
    preserveDrawingBuffer: false,
    powerPreference: "low-power",
  });

  if (!gl) {
    console.error("Failed to get WebGL context");
    return;
  }

  // Immediately clear to the theme's fallback color so the canvas isn't black
  // while the shaders compile
  {
    const { r, g, b } = palette.fallback;
    gl.clearColor(r, g, b, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
  }

  const vertexShaderSrc = `#version 300 es
    in vec2 a_position;
    out vec2 _vs2fs_location0;
    void main() {
      _vs2fs_location0 = a_position * 0.5 + 0.5;
      gl_Position = vec4(a_position, 0.0, 1.0);
    }
  `;

  const vertexShader = createShader(gl, gl.VERTEX_SHADER, vertexShaderSrc);
  const fragmentShader = createShader(
    gl,
    gl.FRAGMENT_SHADER,
    fragmentShaderSrc,
  );
  if (!vertexShader || !fragmentShader) return;

  const program = createProgram(gl, vertexShader, fragmentShader);
  if (!program) return;

  const positionBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
    gl.STATIC_DRAW,
  );

  const positionLocation = gl.getAttribLocation(program, "a_position");
  // Naga emits one std140 Params block. Reflect its size/offsets on the
  // actual GL driver, rather than relying only on WGSL's matching layout.
  const offsets: Record<string, number> = {
    time: 0,
    resolution: 8,
    mouse: 16,
    is_dark: 24,
    oled: 28,
    num_bands: 32,
    "colors[0]": 48,
  };
  if (
    gl.getProgramParameter(program, gl.ACTIVE_UNIFORM_BLOCKS) !== 1 ||
    gl.getActiveUniformBlockParameter(
      program,
      0,
      gl.UNIFORM_BLOCK_DATA_SIZE,
    ) !== 176
  ) {
    throw new Error("Generated background uniform block layout changed");
  }
  const indices = Array.from(
    gl.getActiveUniformBlockParameter(
      program,
      0,
      gl.UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
    ) as Uint32Array,
  );
  const actualOffsets = gl.getActiveUniforms(
    program,
    indices,
    gl.UNIFORM_OFFSET,
  ) as number[];
  for (const [i, index] of indices.entries()) {
    const name = gl.getActiveUniform(program, index)!.name.split(".").pop()!;
    if (offsets[name] !== actualOffsets[i])
      throw new Error("Generated background uniform offset changed: " + name);
  }
  const uniforms = new Float32Array(44);
  const uniformBuffer = gl.createBuffer();
  gl.bindBuffer(gl.UNIFORM_BUFFER, uniformBuffer);
  gl.bufferData(gl.UNIFORM_BUFFER, uniforms.byteLength, gl.DYNAMIC_DRAW);
  gl.uniformBlockBinding(program, 0, 0);
  gl.bindBufferBase(gl.UNIFORM_BUFFER, 0, uniformBuffer);

  let elapsedTime = 0;
  let lastFrameTime = performance.now();
  const targetSpeed = 0;
  let speed = 0.09; // Start with bump-like energy for initial lava-lamp effect
  let isAnimating = true;

  // The decay still to come from speed s moves the scene by 2000·s time units,
  // which below this is under one canvas pixel even on a 1.5x desktop canvas —
  // drawing it would burn frames on motion nobody can see.
  const SPEED_THRESHOLD = 0.005;
  const COLOR_THRESHOLD = 0.001;

  const initialColorData = palette;
  let targetColors = initialColorData.colors;
  let currentColors = [...targetColors]; // Start with target colors
  let numBands = initialColorData.num_bands;

  // Mouse-driven sun anchor (normalized 0..1, y is up). Default is horizontally
  // centered so touch / no-mouse users see the sun behind the landing composition.
  const DEFAULT_MOUSE = [0.5, 0.4];
  let targetMouse: number[] = [...DEFAULT_MOUSE];
  let currentMouse: number[] = [...DEFAULT_MOUSE];
  const MOUSE_THRESHOLD = 0.0005;

  function render() {
    if (!canvas || !gl) return;

    const now = performance.now();
    const deltaTime = now - lastFrameTime;
    lastFrameTime = now;

    // Decay speed back to target using zeno
    speed = zeno(speed, targetSpeed, deltaTime / 1000, 0.5);

    // Interpolate colors towards target
    currentColors = zeno(currentColors, targetColors, deltaTime / 1000, 18.0);

    // Interpolate the sun toward the mouse cursor (or fall back to default)
    currentMouse = zeno(currentMouse, targetMouse, deltaTime / 1000, 20.0);

    elapsedTime += deltaTime * speed;

    gl.useProgram(program);

    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.enableVertexAttribArray(positionLocation);
    gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);

    uniforms[0] = elapsedTime;
    uniforms[2] = canvas.width;
    uniforms[3] = canvas.height;
    uniforms[4] = currentMouse[0];
    uniforms[5] = currentMouse[1];
    uniforms[6] = currentTheme === "light" ? 0 : 1;
    uniforms[7] = currentTheme === "oled" ? 1 : 0;
    uniforms[8] = numBands;
    for (let i = 0; i < numBands; i++) {
      uniforms.set(currentColors.slice(i * 3, i * 3 + 3), 12 + i * 4);
    }
    gl.bindBuffer(gl.UNIFORM_BUFFER, uniformBuffer);
    gl.bufferSubData(gl.UNIFORM_BUFFER, 0, uniforms);

    gl.drawArrays(gl.TRIANGLES, 0, 6);

    // Check if animation has fully settled (speed, colors, and mouse all converged)
    if (speed < SPEED_THRESHOLD) {
      let colorsSettled = true;
      for (let i = 0; i < currentColors.length; i++) {
        if (Math.abs(currentColors[i] - targetColors[i]) > COLOR_THRESHOLD) {
          colorsSettled = false;
          break;
        }
      }
      const mouseSettled =
        Math.abs(currentMouse[0] - targetMouse[0]) < MOUSE_THRESHOLD &&
        Math.abs(currentMouse[1] - targetMouse[1]) < MOUSE_THRESHOLD;
      if (colorsSettled && mouseSettled) {
        speed = 0;
        isAnimating = false;
        animationFrameId = null;
        return;
      }
    }

    animationFrameId = requestAnimationFrame(render);
  }

  // Restart the animation loop if it has settled
  function ensureAnimating() {
    if (!isAnimating) {
      isAnimating = true;
      lastFrameTime = performance.now();
      render();
    }
  }

  // Expose updateColors for theme changes
  (
    self as typeof self & {
      updateShaderColors?: (palette: BackgroundPalette) => void;
      bumpSpeed?: (multiplier?: number) => void;
    }
  ).updateShaderColors = (newColorData) => {
    targetColors = newColorData.colors;
    numBands = newColorData.num_bands;
    ensureAnimating();
  };

  // Expose ensureAnimating for resize redraws
  (
    self as typeof self & {
      updateShaderColors?: (palette: BackgroundPalette) => void;
      bumpSpeed?: (multiplier?: number) => void;
      ensureAnimating?: () => void;
    }
  ).ensureAnimating = ensureAnimating;

  // Expose bumpSpeed function
  (
    self as typeof self & {
      updateShaderColors?: (palette: BackgroundPalette) => void;
      bumpSpeed?: (multiplier?: number) => void;
    }
  ).bumpSpeed = (multiplier = 3.0) => {
    speed = 0.03 * multiplier;
    ensureAnimating();
  };

  // Expose setMouse for main-thread cursor updates. Always cache the latest
  // position so a theme switch into dark/oled picks it up immediately, but
  // only wake the render loop when the active shader actually reads u_mouse.
  (
    self as typeof self & {
      setMouse?: (x: number, y: number) => void;
    }
  ).setMouse = (x: number, y: number) => {
    targetMouse = [x, y];
    if (currentTheme === "dark" || currentTheme === "oled") {
      ensureAnimating();
    }
  };

  render();
  self.postMessage({ type: "ready" });
}

// Listen for messages from the main thread
self.addEventListener("message", (event: MessageEvent<WorkerMessage>) => {
  const {
    type,
    canvas: offscreenCanvas,
    theme,
    palette,
    width,
    height,
    devicePixelRatio,
  } = event.data;

  switch (type) {
    case "init": {
      if (
        offscreenCanvas &&
        theme &&
        palette &&
        width !== undefined &&
        height !== undefined &&
        devicePixelRatio !== undefined
      ) {
        initWebGL(
          offscreenCanvas,
          theme,
          palette,
          width,
          height,
          devicePixelRatio,
        );
      }
      break;
    }

    case "resize": {
      if (
        canvas &&
        gl &&
        width !== undefined &&
        height !== undefined &&
        devicePixelRatio !== undefined
      ) {
        fit(width, height, devicePixelRatio);
        const ensureAnim = (
          self as typeof self & { ensureAnimating?: () => void }
        ).ensureAnimating;
        if (ensureAnim) {
          ensureAnim();
        }
      }
      break;
    }

    case "theme": {
      if (theme && palette) {
        currentTheme = theme;
        const updateColors = (
          self as typeof self & {
            updateShaderColors?: (palette: BackgroundPalette) => void;
          }
        ).updateShaderColors;
        if (updateColors) {
          updateColors(palette);
        }
      }
      break;
    }

    case "stop": {
      if (animationFrameId !== null) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = null;
      }
      break;
    }

    case "bump": {
      const bumpSpeed = (
        self as typeof self & { bumpSpeed?: (multiplier?: number) => void }
      ).bumpSpeed;
      if (bumpSpeed) {
        bumpSpeed(event.data.multiplier);
      }
      break;
    }

    case "mouse": {
      const setMouse = (
        self as typeof self & {
          setMouse?: (x: number, y: number) => void;
        }
      ).setMouse;
      if (
        setMouse &&
        event.data.x !== undefined &&
        event.data.y !== undefined
      ) {
        setMouse(event.data.x, event.data.y);
      }
      break;
    }

    default: {
      console.warn("[Worker] Unknown message type:", type);
      break;
    }
  }
});
