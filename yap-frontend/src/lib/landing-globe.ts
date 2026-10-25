/**
 * The landing hero's little world: a cartoon Earth, slowly turning, with
 * every course's landmark standing at (or, where they crowd, near) its real
 * place like a pop-up cutout, and a few clouds drifting round. The
 * coastlines are Natural Earth's (via world-atlas), loaded with the art.
 * Drawn in 2D in the course icons' style (ink outlines, flat fills printed
 * slightly off register), with real 3D
 * placement: an orthographic sphere whose pole leans toward the viewer, so
 * cutouts on the far side poke up over the horizon and sink behind it.
 *
 * `mountGlobe` starts it and returns a cleanup. The loop only runs while the
 * canvas is on screen and the tab is visible; under `prefers-reduced-motion`
 * a single frame is drawn instead. It can be dragged to spin it. The canvas
 * gets `data-ready` once the art has loaded, so CSS can fade it in.
 *
 * The canvas paints the whole hero (the night sky goes edge to edge), but
 * where the world sits is the page layout's call: it fills `slot`, a square
 * element laid out with the copy.
 */

import {
  type GeoPermissibleObjects,
  geoCircle,
  geoContains,
  geoOrthographic,
  geoPath,
} from "d3-geo";
import type {
  GeometryCollection,
  MultiPolygon,
  Topology,
} from "topojson-specification";

const INK = "#1E2240";
const SEA = "#6FB9D6";
const SEA_SHIFT = "#86A9E0";
const LAND = "#8FCB8B";
const LAND_SHIFT = "#4FAE73";
/** The cloud from the icons, in their 128-unit space. */
const CLOUD = new Path2D(
  "M16 48 Q15 40 23 41 Q26 34 34 37 Q41 36 41 44 Q46 45 44 50 H19 Q15 50 16 48Z",
);
/** How far the pole leans toward the viewer, radians. */
const TILT = 0.56;
/** The closest two landmarks may stand, radians apart on the sphere. */
const GAP = 0.3;
/** Turning speed, radians per second. */
const SPIN = 0.12;
const DEG = Math.PI / 180;

/** A landmark's cutout art and where on Earth it stands, in degrees. */
export interface Landmark {
  url: string;
  lat: number;
  lon: number;
}

type Vec3 = [number, number, number];

function mulberry(seed: number) {
  let s = seed;
  return () => {
    s = (s + 0x6d2b79f5) | 0;
    let t = Math.imul(s ^ (s >>> 15), 1 | s);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/** A unit vector at (lat, lon) after the spin and the tilt: x right, y up,
 *  z toward the viewer. */
function project(lat: number, lon: number, spin: number, r = 1): Vec3 {
  const x = Math.cos(lat) * Math.sin(lon + spin);
  const y = Math.sin(lat);
  const z = Math.cos(lat) * Math.cos(lon + spin);
  return [
    x * r,
    (y * Math.cos(TILT) - z * Math.sin(TILT)) * r,
    (y * Math.sin(TILT) + z * Math.cos(TILT)) * r,
  ];
}

/** The icon's art cropped to its opaque pixels, rasterised once. */
async function loadCutout(url: string) {
  const img = new Image();
  img.src = url;
  await img.decode();
  const size = 384;
  const full = document.createElement("canvas");
  full.width = full.height = size;
  const fctx = full.getContext("2d", { willReadFrequently: true })!;
  fctx.drawImage(img, 0, 0, size, size);
  const { data } = fctx.getImageData(0, 0, size, size);
  let [x0, y0, x1, y1] = [size, size, 0, 0];
  for (let y = 0; y < size; y++)
    for (let x = 0; x < size; x++)
      if (data[(y * size + x) * 4 + 3] > 8) {
        x0 = Math.min(x0, x);
        x1 = Math.max(x1, x);
        y0 = Math.min(y0, y);
        y1 = Math.max(y1, y);
      }
  const out = document.createElement("canvas");
  out.width = x1 - x0 + 1;
  out.height = y1 - y0 + 1;
  out.getContext("2d")!.drawImage(full, -x0, -y0);
  return out;
}

/** Landmarks at their real places, nudged apart until no two stand closer
 *  than `gap` radians (each held near home by a spring), so a crowded
 *  region like western Europe fans out instead of stacking up. Takes and
 *  returns [lat, lon] in radians. */
function spread(homes: [number, number][], gap: number) {
  const vec = ([lat, lon]: [number, number]): Vec3 => [
    Math.cos(lat) * Math.sin(lon),
    Math.sin(lat),
    Math.cos(lat) * Math.cos(lon),
  ];
  const home = homes.map(vec);
  const pos = home.map((v) => [...v] as Vec3);
  for (let step = 0; step < 300; step++) {
    for (let i = 0; i < pos.length; i++)
      for (let j = i + 1; j < pos.length; j++) {
        const d = pos[i].map((c, k) => c - pos[j][k]);
        const dist = Math.hypot(...d);
        if (dist >= gap) continue;
        const push = ((gap - dist) / 2 / dist) * 0.5;
        for (let k = 0; k < 3; k++) {
          pos[i][k] += d[k] * push;
          pos[j][k] -= d[k] * push;
        }
      }
    for (let i = 0; i < pos.length; i++) {
      for (let k = 0; k < 3; k++) pos[i][k] += (home[i][k] - pos[i][k]) * 0.02;
      const n = Math.hypot(...pos[i]);
      for (let k = 0; k < 3; k++) pos[i][k] /= n;
    }
  }
  return pos.map(([x, y, z]): [number, number] => [Math.asin(y), Math.atan2(x, z)]);
}

/** Elevation bands, lowest first, each drawn over the one before like
 *  layers of paper. The shelf is the shallow sea round the coasts; the rest
 *  are clipped to the land. `assets/relief.json` was made once from NOAA's
 *  ETOPO1 sampled every half degree: d3-contour at -200, 500, 1500 and
 *  3000 m, specks under 1e-3 sr dropped, then topojson-simplify at the 0.9
 *  quantile and quantized to 4e3. */
const RELIEF = [
  { band: "shelf", fill: "#9DD3E6", shift: "#86C3DC" },
  { band: "low", fill: "#C2DD8C", shift: "#9FC56E" },
  { band: "high", fill: "#EBCB95", shift: "#D4A56A" },
  { band: "peak", fill: "#FFF7EC", shift: "#DCCFC0" },
] as const;

async function loadEarth() {
  const [{ feature }, landTopo, reliefTopo] = await Promise.all([
    import("topojson-client"),
    import("world-atlas/land-110m.json"),
    import("@/assets/relief.json"),
  ]);
  const land = landTopo.default as unknown as Topology<{
    land: GeometryCollection;
  }>;
  const relief = reliefTopo.default as unknown as Topology<
    Record<(typeof RELIEF)[number]["band"], MultiPolygon>
  >;
  return {
    land: feature(land, land.objects.land),
    relief: RELIEF.map((r) => ({
      ...r,
      shape: feature(relief, relief.objects[r.band]),
    })),
  };
}

function buildWorld(
  landmarks: Landmark[],
  art: HTMLCanvasElement[],
  { land, relief }: Awaited<ReturnType<typeof loadEarth>>,
) {
  const rnd = mulberry(3);
  const placed = spread(
    landmarks.map((l) => [l.lat * DEG, l.lon * DEG]),
    GAP,
  );
  const sites = placed.map(([lat, lon], i) => ({ img: art[i], lat, lon }));
  // A landmark pushed out to sea stands on an island of its own.
  const islands = placed
    .map(([lat, lon]) => [lon / DEG, lat / DEG] as [number, number])
    .filter((at) => !geoContains(land, at))
    .map((at) => geoCircle().center(at).radius(4)());
  const clouds = Array.from({ length: 6 }, (_, i) => ({
    lat: (8 + rnd() * 50) * DEG,
    lon: (i / 6) * Math.PI * 2 + 0.4,
    speed: 0.02 + rnd() * 0.03,
    alt: 1.18 + rnd() * 0.18,
    size: 0.7 + rnd() * 0.5,
  }));
  const stars = Array.from({ length: 140 }, () => [rnd(), rnd(), rnd()]);
  // The projection for the coastlines; its rotation is set per frame to
  // match `project`.
  // No adaptive resampling: the data is dense enough without it, and it is
  // most of the projection's cost.
  const projection = geoOrthographic().precision(0);
  return {
    sites,
    clouds,
    stars,
    land: {
      type: "FeatureCollection" as const,
      features: [
        ...land.features,
        ...islands.map((geometry) => ({
          type: "Feature" as const,
          properties: {},
          geometry,
        })),
      ],
    },
    shelf: relief[0],
    relief: relief.slice(1),
    projection,
    path: geoPath(projection),
  };
}

function drawWorld(
  ctx: CanvasRenderingContext2D,
  canvas: HTMLCanvasElement,
  slot: HTMLElement,
  world: ReturnType<typeof buildWorld>,
  t: number,
  spin: number,
) {
  const dpr = window.devicePixelRatio || 1;
  const w = canvas.clientWidth;
  const h = canvas.clientHeight;
  const pw = Math.max(1, Math.round(w * dpr));
  const ph = Math.max(1, Math.round(h * dpr));
  if (canvas.width !== pw || canvas.height !== ph) {
    // (aliased: the lint config forbids assigning to a parameter's properties)
    const el = canvas;
    el.width = pw;
    el.height = ph;
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, w, h);
  const night =
    getComputedStyle(canvas).getPropertyValue("--fx-night").trim() === "1";

  // The planet takes the middle of the slot, leaving room round it for
  // the cutouts and clouds.
  const box = canvas.getBoundingClientRect();
  const at = slot.getBoundingClientRect();
  const R = Math.min(at.width, at.height) * 0.33;
  const cx = at.left - box.left + at.width / 2;
  const cy = at.top - box.top + at.height * 0.56;
  const lw = Math.max(2, R * 0.016);
  const [sx, sy] = [R * 0.02, R * 0.016]; // the off-register fill
  const screen = (p: Vec3): [number, number] => [cx + p[0] * R, cy - p[1] * R];

  if (night) {
    for (const [x, y, k] of world.stars) {
      const a = 0.3 + 0.45 * Math.sin(t * (0.6 + k) + k * 40) ** 2;
      ctx.fillStyle = `rgba(247,239,246,${a.toFixed(3)})`;
      ctx.beginPath();
      ctx.arc(x * w, y * h, 0.6 + k * 1.1, 0, Math.PI * 2);
      ctx.fill();
    }
  }
  const glow = ctx.createRadialGradient(cx, cy, R * 0.9, cx, cy, R * 1.8);
  glow.addColorStop(
    0,
    night ? "rgba(226,115,223,0.28)" : "rgba(185,60,182,0.12)",
  );
  glow.addColorStop(1, "rgba(226,115,223,0)");
  ctx.fillStyle = glow;
  ctx.fillRect(0, 0, w, h);

  // Everything standing off the surface, back to front: what is behind the
  // planet is drawn before it, so only the part above the horizon shows.
  const items = [
    ...world.sites.map((s) => {
      const p = project(s.lat, s.lon, spin);
      return { p, draw: () => drawCutout(s.img, p) };
    }),
    ...world.clouds.map((c) => {
      const p = project(c.lat, c.lon + t * c.speed, spin, c.alt);
      return { p, draw: () => drawCloud(p, c.size * depth(p[2] / c.alt)) };
    }),
  ].sort((a, b) => a.p[2] - b.p[2]);

  /** Things facing us draw a little larger than things at the horizon, so
   *  the turn reads as depth. Takes z on the unit sphere. */
  const depth = (z: number) => 0.7 + 0.3 * z;

  function drawCutout(img: HTMLCanvasElement, p: Vec3) {
    const [x, y] = screen(p);
    // Cutouts stand up on screen, leaning outward only toward the left and
    // right rims, where they would otherwise stick into the planet; the
    // true surface normal would lay them flat or hang them upside down.
    const len = Math.hypot(p[0], p[1]);
    const angle = 0.7 * p[0] * len;
    const H = R * 0.32 * depth(p[2]);
    const W = (H * img.width) / img.height;
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(angle);
    ctx.drawImage(img, -W / 2, -H + R * 0.02, W, H);
    ctx.restore();
  }

  function drawCloud(p: Vec3, size: number) {
    const [x, y] = screen(p);
    const k = (R / 150) * size;
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(k, k);
    ctx.translate(-30, -43);
    ctx.save();
    ctx.translate(sx / k, sy / k);
    ctx.fillStyle = "#FFFFFF";
    ctx.fill(CLOUD);
    ctx.restore();
    ctx.lineWidth = lw / k;
    ctx.strokeStyle = INK;
    ctx.stroke(CLOUD);
    ctx.restore();
  }

  for (const it of items) if (it.p[2] < 0) it.draw();

  // The planet: sea, land, the shaded side, then the outline.
  ctx.save();
  ctx.beginPath();
  ctx.arc(cx, cy, R, 0, Math.PI * 2);
  ctx.clip();
  ctx.fillStyle = SEA_SHIFT;
  ctx.fillRect(cx - R, cy - R, 2 * R, 2 * R);
  ctx.fillStyle = SEA;
  ctx.beginPath();
  ctx.arc(cx + sx, cy + sy, R, 0, Math.PI * 2);
  ctx.fill();
  // d3 rotates the point (-λ, -φ) to the centre; ours is (-spin, TILT).
  world.projection
    .scale(R)
    .translate([cx, cy])
    .rotate([spin / DEG, -TILT / DEG]);
  // Each shape is projected once per frame and then filled, offset and
  // stroked from the same path.
  const trace = (shape: GeoPermissibleObjects) => {
    const out = new Path2D();
    // (d3's context type asks for beginPath, which Path2D lacks and d3
    // never calls)
    world.path.context(Object.assign(out, { beginPath() {} }))(shape);
    return out;
  };
  /** A flat fill printed off register, as in the icons. */
  const layer = (shape: Path2D, fill: string, shift: string) => {
    ctx.fillStyle = shift;
    ctx.fill(shape);
    ctx.save();
    ctx.translate(sx, sy);
    ctx.fillStyle = fill;
    ctx.fill(shape);
    ctx.restore();
  };
  ctx.lineJoin = "round";
  layer(trace(world.shelf.shape), world.shelf.fill, world.shelf.shift);
  const land = trace(world.land);
  layer(land, LAND, LAND_SHIFT);
  ctx.save();
  ctx.clip(land);
  // Contours in a lighter ink than the coast, so the coast still leads.
  ctx.lineWidth = lw * 0.3;
  ctx.strokeStyle = "rgba(30,34,64,0.5)";
  for (const r of world.relief) {
    const band = trace(r.shape);
    layer(band, r.fill, r.shift);
    ctx.stroke(band);
  }
  ctx.restore();
  ctx.lineWidth = lw * 0.6;
  ctx.strokeStyle = INK;
  ctx.stroke(land);
  ctx.fillStyle = "rgba(30,34,64,0.16)";
  ctx.beginPath();
  ctx.arc(cx, cy, R * 1.02, 0, Math.PI * 2);
  ctx.arc(cx - R * 0.16, cy - R * 0.14, R * 0.98, 0, Math.PI * 2, true);
  ctx.fill();
  ctx.restore();
  ctx.lineWidth = lw;
  ctx.beginPath();
  ctx.arc(cx, cy, R, 0, Math.PI * 2);
  ctx.stroke();

  for (const it of items) if (it.p[2] >= 0) it.draw();
  return { cx, cy, R };
}

export function mountGlobe(
  canvas: HTMLCanvasElement,
  slot: HTMLElement,
  landmarks: Landmark[],
): () => void {
  const ctx = canvas.getContext("2d");
  if (!ctx) return () => {};
  let disposed = false;
  let frame = 0;
  let cleanup = () => {};

  void Promise.all([
    Promise.all(landmarks.map((l) => loadCutout(l.url))),
    loadEarth(),
  ]).then(([art, earth]) => {
    if (disposed) return;
    const world = buildWorld(landmarks, art, earth);
    // The spin is integrated rather than derived from the clock, so a drag
    // can fling it and it settles back to its own pace.
    let spin = 0;
    let speed = SPIN;
    let last = performance.now();
    let disc = { cx: 0, cy: 0, R: 0 };
    const draw = (now: number) => {
      const dt = Math.min(0.1, (now - last) / 1000);
      last = now;
      if (!drag) {
        speed += (SPIN - speed) * Math.min(1, dt * 1.5);
        spin += speed * dt;
      }
      disc = drawWorld(ctx, canvas, slot, world, now / 1000, spin);
    };
    canvas.setAttribute("data-ready", "");

    let drag: { x: number; t: number } | null = null;
    const onDown = (e: PointerEvent) => {
      const r = canvas.getBoundingClientRect();
      const d = Math.hypot(
        e.clientX - r.left - disc.cx,
        e.clientY - r.top - disc.cy,
      );
      if (d > disc.R * 1.3) return;
      drag = { x: e.clientX, t: e.timeStamp };
      canvas.setPointerCapture(e.pointerId);
    };
    const onMove = (e: PointerEvent) => {
      if (!drag) return;
      const delta = (e.clientX - drag.x) / disc.R;
      spin += delta;
      speed = delta / Math.max(0.008, (e.timeStamp - drag.t) / 1000);
      drag = { x: e.clientX, t: e.timeStamp };
      if (!frame) draw(performance.now());
    };
    const onUp = () => {
      drag = null;
    };
    canvas.addEventListener("pointerdown", onDown);
    canvas.addEventListener("pointermove", onMove);
    canvas.addEventListener("pointerup", onUp);
    canvas.addEventListener("pointercancel", onUp);

    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      speed = 0;
      draw(last);
      const ro = new ResizeObserver(() => draw(performance.now()));
      ro.observe(canvas);
      cleanup = () => ro.disconnect();
      return;
    }

    let visible = false;
    const loop = (now: number) => {
      draw(now);
      frame = requestAnimationFrame(loop);
    };
    const sync = () => {
      const run = visible && document.visibilityState === "visible";
      if (run && !frame) {
        last = performance.now();
        frame = requestAnimationFrame(loop);
      }
      if (!run && frame) {
        cancelAnimationFrame(frame);
        frame = 0;
      }
    };
    const observer = new IntersectionObserver((entries) => {
      visible = entries.some((e) => e.isIntersecting);
      sync();
    });
    observer.observe(canvas);
    document.addEventListener("visibilitychange", sync);
    draw(last);
    cleanup = () => {
      observer.disconnect();
      document.removeEventListener("visibilitychange", sync);
      canvas.removeEventListener("pointerdown", onDown);
      canvas.removeEventListener("pointermove", onMove);
      canvas.removeEventListener("pointerup", onUp);
      canvas.removeEventListener("pointercancel", onUp);
    };
  });

  return () => {
    disposed = true;
    if (frame) cancelAnimationFrame(frame);
    cleanup();
  };
}
