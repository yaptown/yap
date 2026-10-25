import type { ReaderLayout } from "../../../yap-frontend-rs/pkg";

export type PageSnapshot = { flow: HTMLElement; page: number };

/** A sheet partway through turning. A finger drags it; a tap settles it from the start. */
export type PageTurn = {
  /** Holds the sheet `progress` of the way through the turn, from 0 to 1. */
  drag: (progress: number) => void;
  /** Animates the rest of the way over, or back if not `complete`, then calls `done` and removes the sheet. */
  settle: (complete: boolean, done: () => void) => void;
  /** Removes the sheet at once. */
  cancel: () => void;
};

/** Temporary, aria-hidden DOM copies bend the actual typeset page. The live text
 * remains the accessible document; no rasterization, fonts, or network requests. */
export function startPageTurn(
  host: HTMLElement,
  flow: HTMLElement,
  layout: ReaderLayout,
  before: PageSnapshot,
  after: PageSnapshot,
  direction: 1 | -1,
): PageTurn {
  const width = host.clientWidth;
  const height = host.clientHeight;
  const spread = layout.page_width * layout.columns + layout.gap * (layout.columns - 1);
  const stride = spread + layout.gap;
  const margin = (width - spread) / 2;
  const double = layout.columns === 2;
  const sheet = double ? width / 2 : width;
  const spine = double ? sheet : 0;
  const forward = direction === 1;
  const lower = forward ? before : after;
  const upper = forward ? after : before;
  const paper = getComputedStyle(host).getPropertyValue("--reader-paper").trim();
  const layer = document.createElement("div");
  layer.setAttribute("aria-hidden", "true");
  layer.inert = true;
  Object.assign(layer.style, {
    position: "absolute", inset: "0", pointerEvents: "none", overflow: "hidden",
    background: paper, perspective: `${width * 3}px`, perspectiveOrigin: `${double ? spine : width / 2}px 50%`,
  });
  const snapshot = (source: PageSnapshot, offset: number) => {
    const clone = source.flow.cloneNode(true) as HTMLElement;
    clone.removeAttribute("id");
    clone.querySelectorAll("[id]").forEach((node) => node.removeAttribute("id"));
    Object.assign(clone.style, {
      position: "absolute", left: `${margin - offset}px`, top: `${layout.top}px`,
      transform: `translateX(${-source.page * stride}px)`, transition: "none", visibility: "visible",
    });
    return clone;
  };
  const fixedPage = (page: PageSnapshot, left: number, pageWidth: number) => {
    const clip = document.createElement("div");
    Object.assign(clip.style, { position: "absolute", left: `${left}px`, top: "0", width: `${pageWidth}px`, height: "100%", overflow: "hidden", background: paper });
    clip.append(snapshot(page, left));
    layer.append(clip);
  };
  if (double) {
    fixedPage(lower, 0, sheet);
    fixedPage(upper, sheet, sheet);
  } else fixedPage(upper, 0, width);

  const shadow = document.createElement("div");
  Object.assign(shadow.style, { position: "absolute", top: "0", bottom: "0", left: `${spine}px`, width: `${double ? sheet : Math.min(64, width * 0.12)}px`, background: "linear-gradient(90deg, #0008, #0000)", opacity: "0" });
  layer.append(shadow);
  const count = 28;
  const segment = sheet / count;
  const strips = Array.from({ length: count }, (_, index) => {
    const strip = document.createElement("div");
    Object.assign(strip.style, { position: "absolute", top: "0", left: `${spine}px`, width: `${segment + 2}px`, height: `${height}px`, transformOrigin: "0 50%", transformStyle: "preserve-3d", willChange: "transform" });
    // A single page sweeps out of the viewport with its front facing us. Only
    // a two-page spread turns over far enough to need the reverse of the leaf.
    for (const back of double ? [false, true] : [false]) {
      const face = document.createElement("div");
      Object.assign(face.style, { position: "absolute", inset: "0", overflow: "hidden", backfaceVisibility: "hidden", background: paper, transform: back ? "rotateY(180deg)" : "none" });
      // On the reverse, distance from the spine runs right-to-left.
      const offset = back ? sheet - (index + 1) * segment : spine + index * segment;
      face.append(snapshot(back ? upper : lower, offset));
      const shade = document.createElement("div");
      Object.assign(shade.style, { position: "absolute", inset: "0", background: "#000", opacity: "0" });
      face.append(shade);
      strip.append(face);
    }
    layer.append(strip);
    return strip;
  });
  host.append(layer);
  flow.style.setProperty("visibility", "hidden");
  let frame = 0;
  let progress = 0;
  let stopped = false;
  const cancel = () => {
    stopped = true;
    cancelAnimationFrame(frame);
    layer.remove();
    flow.style.removeProperty("visibility");
  };
  const draw = () => {
    const t = forward ? progress : 1 - progress;
    let x = double ? 0 : -sheet * t;
    let z = 0;
    strips.forEach((strip, index) => {
      const u = (index + 0.5) / count;
      // Vary the tangent across the sheet; integrate it to keep neighbouring
      // strips joined. A rigid rotateY would have the same angle everywhere.
      // In single-page mode, a shallow arch lifts the middle of the sheet.
      // Its tangents never reach 90 degrees, so text stays front-facing while
      // the next page remains still underneath. Reverse turns retrace it.
      const angle = double
        ? Math.PI * t + 1.5 * Math.sin(Math.PI * t) * (u - 0.5)
        : 0.65 * Math.sin(Math.PI * t) * Math.cos(Math.PI * u);
      strip.style.setProperty("transform", `translate3d(${x}px,0,${z}px) rotateY(${-angle}rad)`);
      const darkness = Math.abs(Math.sin(angle)) * 0.16;
      for (const face of strip.children) (face.lastElementChild as HTMLElement).style.opacity = String(darkness);
      x += segment * Math.cos(angle);
      z += segment * Math.sin(angle);
    });
    if (!double) shadow.style.transform = `translateX(${x}px)`;
    shadow.style.opacity = String(Math.sin(Math.PI * t) * (double ? 0.3 : 0.4));
  };
  draw();
  return {
    drag: (next) => {
      progress = Math.min(1, Math.max(0, next));
      draw();
    },
    settle: (complete, done) => {
      cancelAnimationFrame(frame);
      const from = progress;
      const to = complete ? 1 : 0;
      // A whole turn takes the full time; what a finger has already done is skipped.
      const duration = Math.max(160, Math.abs(to - from) * (double ? 620 : 480));
      let start: number | undefined;
      const step = (now: number) => {
        start ??= now;
        const elapsed = Math.min(1, (now - start) / duration);
        // Ease out, carrying on from the finger's motion (or a tap's flick).
        progress = from + (to - from) * (1 - (1 - elapsed) ** 3);
        draw();
        if (elapsed < 1) frame = requestAnimationFrame(step);
        else if (!stopped) { done(); cancel(); }
      };
      frame = requestAnimationFrame(step);
    },
    cancel,
  };
}
