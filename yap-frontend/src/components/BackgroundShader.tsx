import { useEffect, useRef, useMemo, memo } from "react";
import { useTheme } from "./theme-provider";

function BackgroundShaderComponent() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
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

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const gl = canvas.getContext("webgl", {
      alpha: false,
      antialias: false,
      depth: false,
      stencil: false,
      preserveDrawingBuffer: false,
      powerPreference: "high-performance",
    });
    if (!gl) return;

    // CPU-side LCH to RGB conversion
    function lchToRgb(
      L: number,
      C: number,
      H: number
    ): [number, number, number] {
      // LCH to Lab
      const a = C * Math.cos(H);
      const b = C * Math.sin(H);

      // Lab to XYZ
      const D65 = [0.95047, 1.0, 1.08883];
      const labFInv = (t: number) =>
        t > 0.206893 ? t * t * t : (t - 16.0 / 116.0) / 7.787;

      const fy = (L + 16.0) / 116.0;
      const fx = a / 500.0 + fy;
      const fz = fy - b / 200.0;

      const xyz = [
        D65[0] * labFInv(fx),
        D65[1] * labFInv(fy),
        D65[2] * labFInv(fz),
      ];

      // XYZ to RGB
      const fromLinear = (c: number) =>
        c <= 0.0031308 ? 12.92 * c : 1.055 * Math.pow(c, 1.0 / 2.4) - 0.055;

      const r = 3.2404542 * xyz[0] - 1.5371385 * xyz[1] - 0.4985314 * xyz[2];
      const g = -0.969266 * xyz[0] + 1.8760108 * xyz[1] + 0.041556 * xyz[2];
      const b2 = 0.0556434 * xyz[0] - 0.2040259 * xyz[1] + 1.0572252 * xyz[2];

      return [
        Math.max(0, Math.min(1, fromLinear(r))),
        Math.max(0, Math.min(1, fromLinear(g))),
        Math.max(0, Math.min(1, fromLinear(b2))),
      ];
    }

    // Pre-calculate colors for each band
    // Shader configuration
    const numBands = actualTheme === "dark" ? 6 : 6;
    const speed = 0.1;
    const lightness = actualTheme === "dark" ? 10.0 : 68.0;
    const chroma = actualTheme === "dark" ? 3.0 : 30.0;
    const lightnessShift = actualTheme === "dark" ? 12.0 : 12.0;
    const hueStart = 3.2;
    const hueRange = -3.0;

    const colors: number[] = [];
    for (let i = 0; i < numBands; i++) {
      const band = i / numBands;
      let H = hueStart + band * hueRange;
      H = H % (2 * Math.PI);
      if (H < 0) H += 2 * Math.PI;

      const L = lightness + (band - 0.5) * lightnessShift;
      const rgb = lchToRgb(L, chroma, H);
      colors.push(rgb[0], rgb[1], rgb[2]);
    }

    const vertexShaderSrc = `
      attribute vec2 a_position;
      varying vec2 v_uv;
      void main() {
        v_uv = a_position * 0.5 + 0.5;
        gl_Position = vec4(a_position, 0.0, 1.0);
      }
    `;

    const fragmentShaderSrc = `
      precision highp float;

      varying vec2 v_uv;
      uniform float u_time;
      uniform vec2 u_resolution;
      uniform float u_numBands;
      uniform vec3 u_colors[16]; // Pre-calculated colors (max 16 bands)

      #define PI 3.14159265359
      #define TAU 6.28318530718

      // Static grain function (doesn't change between frames)
      float grain(vec2 uv) {
        return pow(fract(sin(dot(uv, vec2(12.9898, 78.233))) * 43758.5453), 8.0);
      }

      // === Smooth blob with extended tail ===
      float blob(vec2 uv, vec2 center, float radius) {
        float d = distance(uv, center);
        float t = 1.0 - smoothstep(0.0, radius * 2.0, d);
        return t * t;
      }

      void main() {
        vec2 uv = v_uv;
        float aspect = u_resolution.x / u_resolution.y;

        // Normalize to a consistent coordinate system based on the shorter edge
        // This ensures blobs appear the same size on both portrait and landscape
        vec2 uvAspect;
        if (aspect > 1.0) {
          // Landscape: scale x to fit
          uvAspect = vec2(uv.x * aspect, uv.y);
        } else {
          // Portrait: scale y to fit
          uvAspect = vec2(uv.x, uv.y / aspect);
        }

        float t = u_time;

        // Blob definitions
        const int NUM_BLOBS = 6;
        vec2 basePos[6];
        float radius[6];
        vec2 phase[6];
        float weight[6];

        basePos[0] = vec2(0.3, 0.3);   radius[0] = 0.35; phase[0] = vec2(0.0, 0.5);   weight[0] = 1.0;
        basePos[1] = vec2(0.75, 0.35); radius[1] = 0.32; phase[1] = vec2(1.0, 0.0);   weight[1] = 0.95;
        basePos[2] = vec2(0.5, 0.75);  radius[2] = 0.34; phase[2] = vec2(2.0, 1.5);   weight[2] = 1.0;
        basePos[3] = vec2(0.18, 0.65); radius[3] = 0.3;  phase[3] = vec2(0.5, 2.0);   weight[3] = 0.9;
        basePos[4] = vec2(0.85, 0.8);  radius[4] = 0.32; phase[4] = vec2(1.5, 0.3);   weight[4] = 0.9;
        basePos[5] = vec2(0.12, 0.15); radius[5] = 0.28; phase[5] = vec2(2.2, 1.8);   weight[5] = 0.85;

        // Accumulate grayscale value from all blobs
        float value = 0.0;

        for (int i = 0; i < NUM_BLOBS; i++) {
          vec2 offset = vec2(
            sin(t * 0.0003 + phase[i].x) * 0.14,
            cos(t * 0.00025 + phase[i].y) * 0.14
          );

          vec2 pos = basePos[i] + offset;

          // Apply same aspect ratio correction as uvAspect
          if (aspect > 1.0) {
            pos.x *= aspect;
          } else {
            pos.y /= aspect;
          }

          float influence = blob(uvAspect, pos, radius[i]);
          value += influence * weight[i];
        }

        // Add slow-moving background variation
        float bgWave = sin(uv.x * 2.5 + t * 0.00008) * 0.5 + 0.5;
        bgWave *= sin(uv.y * 2.0 + t * 0.00006) * 0.5 + 0.5;
        float baseVariation = bgWave * 0.4;

        // Blend: use base variation in empty areas, blob value where blobs are
        value = max(value, baseVariation * (1.0 - value * 0.8));
        value = clamp(value, 0.0, 0.99);

        // === QUANTIZE GRAYSCALE ===
        float band = floor(value * u_numBands) / u_numBands;

        int bandIndex = int(band * u_numBands);

        // Use pre-calculated color from uniform array
        // WebGL 1.0 doesn't support dynamic indexing, so we use conditionals
        vec3 color = u_colors[0];
        if (bandIndex == 1) color = u_colors[1];
        else if (bandIndex == 2) color = u_colors[2];
        else if (bandIndex == 3) color = u_colors[3];
        else if (bandIndex == 4) color = u_colors[4];
        else if (bandIndex == 5) color = u_colors[5];
        else if (bandIndex == 6) color = u_colors[6];
        else if (bandIndex == 7) color = u_colors[7];
        else if (bandIndex >= 8) color = u_colors[7]; // Clamp to last color

        // Subtle vignette
        float vignette = 1.0 - smoothstep(0.5, 1.5, length(v_uv - 0.5) * 1.3);
        color *= 0.94 + 0.06 * vignette;

        // Add static grain
        float grainValue = grain(v_uv * u_resolution * 0.15);
        float grainAmount = 0.12; // Adjust this to control grain intensity
        color += (grainValue - 0.5) * grainAmount;

        gl_FragColor = vec4(color, 1.0);
      }
    `;

    function createShader(
      gl: WebGLRenderingContext,
      type: number,
      source: string
    ) {
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
      gl: WebGLRenderingContext,
      vertexShader: WebGLShader,
      fragmentShader: WebGLShader
    ) {
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

    const vertexShader = createShader(gl, gl.VERTEX_SHADER, vertexShaderSrc);
    const fragmentShader = createShader(
      gl,
      gl.FRAGMENT_SHADER,
      fragmentShaderSrc
    );
    if (!vertexShader || !fragmentShader) return;

    const program = createProgram(gl, vertexShader, fragmentShader);
    if (!program) return;

    const positionBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 1, -1, -1, 1, -1, 1, 1, -1, 1, 1]),
      gl.STATIC_DRAW
    );

    const positionLocation = gl.getAttribLocation(program, "a_position");
    const timeLocation = gl.getUniformLocation(program, "u_time");
    const resolutionLocation = gl.getUniformLocation(program, "u_resolution");
    const numBandsLocation = gl.getUniformLocation(program, "u_numBands");
    const colorsLocation = gl.getUniformLocation(program, "u_colors");

    function resize() {
      if (!canvas) return;
      // Reduce resolution for better performance (especially in Chrome)
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      const scale = 0.75; // Render at 75% resolution for performance
      canvas.width = window.innerWidth * dpr * scale;
      canvas.height = window.innerHeight * dpr * scale;
      gl.viewport(0, 0, canvas.width, canvas.height);
    }

    window.addEventListener("resize", resize);
    resize();

    const startTime = performance.now();
    let animationFrameId: number;

    function render() {
      if (!canvas) return;
      const elapsed = (performance.now() - startTime) * speed;

      if (!gl) return;

      gl.useProgram(program);

      gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
      gl.enableVertexAttribArray(positionLocation);
      gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);

      gl.uniform1f(timeLocation, elapsed);
      gl.uniform2f(resolutionLocation, canvas.width, canvas.height);
      gl.uniform1f(numBandsLocation, numBands);
      gl.uniform3fv(colorsLocation, colors);

      gl.drawArrays(gl.TRIANGLES, 0, 6);

      animationFrameId = requestAnimationFrame(render);
    }

    render();

    return () => {
      window.removeEventListener("resize", resize);
      cancelAnimationFrame(animationFrameId);
    };
  }, [actualTheme]);

  return (
    <canvas
      ref={canvasRef}
      className="fixed inset-0 w-full h-full -z-10"
      style={{
        pointerEvents: "none",
        willChange: "contents",
        transform: "translateZ(0)",
      }}
    />
  );
}

export const BackgroundShader = memo(BackgroundShaderComponent);
