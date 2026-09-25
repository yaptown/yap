// Single source for WebGL2 and SwiftUI. UV is bottom-left, on both hosts.
struct Params {
    time: f32,
    resolution: vec2<f32>,
    mouse: vec2<f32>,
    is_dark: f32,
    oled: f32,
    num_bands: f32,
    colors: array<vec4<f32>, 8>,
}
@group(0) @binding(0) var<uniform> params: Params;

// === Smooth blob with extended tail (light/oled themes) ===
fn blob(uv: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
  var d: f32 = distance(uv, center);
  var t: f32 = 1.0 - smoothstep(0.0, radius * 2.0, d);
  return t * t;
}

// Single gentle swell shared by all layers.
fn sharedWave(x: f32, t: f32) -> f32 {
  return 0.04 * sin(x * 4.8 - t * 0.08);
}

// Two-octave sine ridge.
fn layerHeight(x: f32, baseY: f32, amp: f32, freq: f32, phase: f32) -> f32 {
  var h: f32 = sin(x * freq + phase)
          + 0.4 * sin(x * freq * 2.17 + phase * 1.7 + 1.3);
  return baseY + amp * h / 1.4;
}

// Map a 0..1 point into the same aspect-corrected space used by uvAspect,
// so distances are measured on a common coordinate system regardless of
// orientation. Keep everything that feeds into lightDist/facing going
// through this helper.
fn toAspect(p: vec2<f32>, aspect: f32) -> vec2<f32> {
  return select(vec2<f32>(p.x, p.y / aspect), vec2<f32>(p.x * aspect, p.y), aspect > 1.0);
}

// Deterministic pseudo-random in [0, 1) from a 2D coordinate.
fn hash21(point: vec2<f32>) -> f32 {
  var p = fract(point * vec2<f32>(443.897, 441.423));
  p += vec2<f32>(dot(p, p + vec2<f32>(19.19)));
  return fract(p.x * p.y);
}

// Smooth value noise: bilinear interpolation of hash values on a grid,
// with smoothstep-eased interpolation weights.
fn valueNoise(p: vec2<f32>) -> f32 {
  var i: vec2<f32> = floor(p);
  var f: vec2<f32> = fract(p);
  f = f * f * (vec2<f32>(3.0) - 2.0 * f);
  var a: f32 = hash21(i);
  var b: f32 = hash21(i + vec2<f32>(1.0, 0.0));
  var c: f32 = hash21(i + vec2<f32>(0.0, 1.0));
  var d: f32 = hash21(i + vec2<f32>(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// Fractal Brownian motion: 4 octaves of value noise at halving amplitude
// and doubling frequency — smoothly varying low-frequency noise.
fn fbm(point: vec2<f32>) -> f32 {
  var p = point;
  var v: f32 = 0.0;
  var a: f32 = 0.5;
  for (var i: i32 = 0; i < 4; i += 1) {
    v += a * valueNoise(p);
    p *= 2.0;
    a *= 0.5;
  }
  return v;
}

fn background(uv: vec2<f32>, params: Params) -> vec3<f32> {
  var aspect: f32 = params.resolution.x / params.resolution.y;
  var uvAspect: vec2<f32> = toAspect(uv, aspect);

  var t: f32 = params.time;
  var value: f32 = 0.0;

  if (params.is_dark > 0.5) {
    // ===== Dark theme: layered sine mountains lit by the cursor =====
    // params.time accumulates in ms-scale via the render loop's speed factor;
    // convert to a seconds-ish scale for the per-layer drift.
    var iTime: f32 = t * 0.001;

    var baseCol: vec3<f32> = mix(vec3<f32>(14.0, 7.0, 16.0) / 255.0, vec3<f32>(0.0), params.oled);
    var haloCol: vec3<f32> = vec3<f32>(128.0, 46.0, 84.0) / 255.0;
    var hotCol: vec3<f32> = vec3<f32>(226.0, 128.0, 162.0) / 255.0;
    var lineCol: vec3<f32> = vec3<f32>(210.0, 108.0, 146.0) / 255.0;

    var lightAnchor: vec2<f32> = toAspect(params.mouse, aspect);
    var lightDist: f32 = distance(uvAspect, lightAnchor);
    // Tighter falloff for OLED — keeps most of the screen pure black.
    var haloRadius: f32 = mix(1.0, 0.5, params.oled);
    var coreRadius: f32 = mix(0.28, 0.18, params.oled);
    var lightInfluence: f32 = 1.0 - smoothstep(0.0, haloRadius, lightDist);
    lightInfluence *= lightInfluence;
    var lightCore: f32 = 1.0 - smoothstep(0.0, coreRadius, lightDist);
    lightCore = lightCore * lightCore * lightCore;

    var col: vec3<f32> = baseCol * 0.45;

    var baseY: array<f32, 4>; var amp: array<f32, 4>; var freq: array<f32, 4>; var phase: array<f32, 4>;
    var rimDepth: array<f32, 4>; var rimBoost: array<f32, 4>;

    baseY[0] = 0.70; amp[0] = 0.16; freq[0] = 1.4; phase[0] = 0.3 + iTime * 0.07;
    baseY[1] = 0.40; amp[1] = 0.07; freq[1] = 3.6; phase[1] = 1.7 - iTime * 0.11;
    baseY[2] = 0.28; amp[2] = 0.07; freq[2] = 3.0; phase[2] = 0.9 - iTime * 0.05;
    baseY[3] = 0.17; amp[3] = 0.05; freq[3] = 4.6; phase[3] = 4.2 + iTime * 0.13;

    rimDepth[0] = 0.12; rimDepth[1] = 0.07; rimDepth[2] = 0.07; rimDepth[3] = 0.06;
    rimBoost[0] = 0.11; rimBoost[1] = 0.06; rimBoost[2] = 0.07; rimBoost[3] = 0.08;

    for (var i: i32 = 0; i < 4; i += 1) {
      var h: f32 = layerHeight(uv.x, baseY[i], amp[i], freq[i], phase[i]) + sharedWave(uv.x, iTime);

      // Numerical slope of the full ridge (both octaves + shared wave).
      // Cheaper to write than to differentiate by hand and automatically
      // correct if layerHeight or sharedWave changes.
      var eps: f32 = 0.001;
      var hLeft: f32 = layerHeight(uv.x - eps, baseY[i], amp[i], freq[i], phase[i]) + sharedWave(uv.x - eps, iTime);
      var hRight: f32 = layerHeight(uv.x + eps, baseY[i], amp[i], freq[i], phase[i]) + sharedWave(uv.x + eps, iTime);
      var normal: vec2<f32> = normalize(vec2<f32>(-(hRight - hLeft) / (2.0 * eps), 1.0));

      // Direction from the ridge point toward the light (aspect-corrected).
      var toLight: vec2<f32> = normalize(lightAnchor - toAspect(vec2<f32>(uv.x, h), aspect));
      var facing: f32 = max(dot(normal, toLight), 0.0);

      // Rim glow just above the ridge.
      if (uv.y >= h && uv.y < h + rimDepth[i]) {
        var ht: f32 = 1.0 - clamp((uv.y - h) / rimDepth[i], 0.0, 1.0);
        ht = ht * ht * ht;
        var haloFactor: f32 = 0.02 + 1.1 * lightInfluence * facing;
        col += mix(haloCol, hotCol, lightCore * 0.6) * (rimBoost[i] * ht * haloFactor);
      }

      // Mountain body.
      if (uv.y < h) {
        var depthWeight: f32 = 0.4 + 0.6 * (f32(i) / 3.0);
        var gradMul: f32 = 0.75 + 0.25 * clamp((h - uv.y) / 0.25, 0.0, 1.0);
        var body: vec3<f32> = baseCol * gradMul;
        // Warm body-lift is purple-theme only — in OLED the body stays black
        // so only the ridge halos read as the "sun".
        body += haloCol * lightInfluence * 0.08 * depthWeight * (1.0 - params.oled);
        col = body;
      }

      // Thin hairline ridge, only on light-facing slopes.
      var lineFalloff: f32 = lightInfluence * facing;
      lineFalloff *= lineFalloff;
      var lineWidth: f32 = 0.35 / params.resolution.y;
      var lineSoftEdge: f32 = 2.5 / params.resolution.y;
      var lineMask: f32 = 1.0 - smoothstep(lineWidth, lineWidth + lineSoftEdge, abs(uv.y - h));
      col = mix(col, lineCol, lineMask * lineFalloff * 0.38);
    }

    // Faint atmospheric haze: low-frequency fBm that slightly lifts darks,
    // cool in unlit areas and warm near the light.
    var haze: f32 = fbm(uv * 2.5 + vec2<f32>(iTime * 0.02, 0.0));
    var hazeTint: vec3<f32> = mix(baseCol * 0.5, haloCol * 0.35, lightInfluence * 0.5);
    col += hazeTint * haze * 0.05;

    return col;
  }

  // ===== Light / OLED themes: original metaballs =====
  const NUM_BLOBS: i32 = 6;
  var basePos: array<vec2<f32>, 6>;
  var radius: array<f32, 6>;
  var phase: array<vec2<f32>, 6>;
  var weight: array<f32, 6>;

  basePos[0] = vec2<f32>(0.3, 0.3);   radius[0] = 0.35; phase[0] = vec2<f32>(0.0, 0.5);   weight[0] = 1.0;
  basePos[1] = vec2<f32>(0.75, 0.35); radius[1] = 0.32; phase[1] = vec2<f32>(1.0, 0.0);   weight[1] = 0.95;
  basePos[2] = vec2<f32>(0.5, 0.75);  radius[2] = 0.34; phase[2] = vec2<f32>(2.0, 1.5);   weight[2] = 1.0;
  basePos[3] = vec2<f32>(0.18, 0.65); radius[3] = 0.3;  phase[3] = vec2<f32>(0.5, 2.0);   weight[3] = 0.9;
  basePos[4] = vec2<f32>(0.85, 0.8);  radius[4] = 0.32; phase[4] = vec2<f32>(1.5, 0.3);   weight[4] = 0.9;
  basePos[5] = vec2<f32>(0.12, 0.15); radius[5] = 0.28; phase[5] = vec2<f32>(2.2, 1.8);   weight[5] = 0.85;

  for (var i: i32 = 0; i < NUM_BLOBS; i += 1) {
    var offset: vec2<f32> = vec2<f32>(
      sin(t * 0.0003 + phase[i].x) * 0.14,
      cos(t * 0.00025 + phase[i].y) * 0.14
    );

    var pos: vec2<f32> = toAspect(basePos[i] + offset, aspect);

    var influence: f32 = blob(uvAspect, pos, radius[i]);
    value += influence * weight[i];
  }

  // Slow-moving background variation
  var bgWave: f32 = sin(uv.x * 2.5 + t * 0.00008) * 0.5 + 0.5;
  bgWave *= sin(uv.y * 2.0 + t * 0.00006) * 0.5 + 0.5;
  var baseVariation: f32 = bgWave * 0.4;

  value = max(value, baseVariation * (1.0 - value * 0.8));
  value = clamp(value, 0.0, 0.99);

  var band: f32 = floor(value * params.num_bands) / params.num_bands;
  var bandIndex: i32 = i32(band * params.num_bands);

  var color: vec3<f32> = params.colors[0].xyz;
  if (bandIndex == 1) { color = params.colors[1].xyz; }
  else if (bandIndex == 2) { color = params.colors[2].xyz; }
  else if (bandIndex == 3) { color = params.colors[3].xyz; }
  else if (bandIndex == 4) { color = params.colors[4].xyz; }
  else if (bandIndex == 5) { color = params.colors[5].xyz; }
  else if (bandIndex == 6) { color = params.colors[6].xyz; }
  else if (bandIndex == 7) { color = params.colors[7].xyz; }
  else if (bandIndex >= 8) { color = params.colors[7].xyz; }

  var vignette: f32 = 1.0 - smoothstep(0.5, 1.5, length(uv - vec2<f32>(0.5)) * 1.3);
  color *= 0.94 + 0.06 * vignette;

  return color;
}
  
@fragment
fn fragment(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(background(uv, params), 1.0);
}
