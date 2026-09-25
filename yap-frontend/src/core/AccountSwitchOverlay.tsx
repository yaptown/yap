import { useLayoutEffect, useRef } from "react";

/** A modal in the top layer also blocks keyboard input and portalled dialogs. */
export function AccountSwitchOverlay({ active }: { active: boolean }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useLayoutEffect(() => {
    const element = dialog.current;
    if (!active || !element) return;
    // Native inertness blocks controls, but document-level handlers (including
    // Radix's capture-phase Escape listener) would still see these events.
    const block = (event: Event) => {
      event.preventDefault();
      event.stopImmediatePropagation();
    };
    const events = ["keydown", "keyup", "pointerdown", "pointerup", "click"];
    for (const event of events) window.addEventListener(event, block, true);
    element.showModal();
    return () => {
      for (const event of events) window.removeEventListener(event, block, true);
      element.close();
    };
  }, [active]);
  return <dialog
    ref={dialog}
    aria-label="Switching account"
    onCancel={(event) => event.preventDefault()}
    className="fixed inset-0 m-auto border-0 bg-transparent p-6 text-foreground outline-none backdrop:bg-background/70"
  ><p role="status">Loading...</p></dialog>;
}
