import type { PendingReviewSlot } from "../../../../yap-frontend-rs/pkg";

/** Disposable UI state. Rust defines the slot; the host handles storage/codecs. */
export class PendingReview<T> {
  private cleared = false;
  private readonly slot: PendingReviewSlot;
  private readonly decode: (payload: unknown) => T;
  private readonly encode: (state: T) => unknown;

  constructor(
    slot: PendingReviewSlot,
    decode: (payload: unknown) => T,
    encode: (state: T) => unknown = (state) => state,
  ) {
    this.slot = slot;
    this.decode = decode;
    this.encode = encode;
  }

  load(): T | undefined {
    try {
      const raw = localStorage.getItem(this.slot.key);
      if (!raw) return undefined;
      const saved = JSON.parse(raw);
      if (saved.identity === this.slot.identity) return this.decode(saved.payload);
    } catch { /* Unavailable storage or obsolete draft: start afresh. */ }
    this.remove();
    return undefined;
  }

  save(state: T) {
    // Late no-op reducer events (or StrictMode resume) must not resurrect a
    // draft whose completion this mounted challenge already acknowledged.
    if (this.cleared) return;
    try {
      localStorage.setItem(this.slot.key, JSON.stringify({
        identity: this.slot.identity,
        payload: this.encode(state),
      }));
    } catch { /* Storage full or unavailable: the review still works. */ }
  }

  clear() {
    this.cleared = true;
    this.remove();
  }

  private remove() {
    try { localStorage.removeItem(this.slot.key); } catch { /* Storage unavailable. */ }
  }
}
