import type { PointerEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** Anything that should behave as a click rather than a drag handle. */
const INTERACTIVE = "button, input, select, textarea, a, [role='button']";

/**
 * Drags the window from a title-bar-like surface.
 *
 * Replaces `data-tauri-drag-region`, which cannot be used here: Tauri's
 * injected drag script maximises the window on any double-click of a drag
 * region, and it does that regardless of `maximizable: false`
 * (tauri-apps/tauri#12006). A 380px widget has no sensible maximised state,
 * and driving the drag ourselves means there is no double-click behaviour to
 * suppress in the first place.
 *
 * The attribute's "deep" mode let a drag start anywhere in the bar including
 * the gaps between controls, while leaving buttons clickable. That is
 * reproduced here by starting a drag unless the pointer went down on
 * something interactive -- an earlier bug in this project left the drag
 * surface unreachable, so covering the gaps matters.
 *
 * Shared rather than duplicated because there are two such surfaces: the
 * main title bar, and the header of any open panel. Panels are absolutely
 * positioned over the whole card, so while one is open the main title bar is
 * buried and its header is the only thing left to drag from.
 */
export async function startWindowDrag(e: PointerEvent) {
  if (e.button !== 0) return;
  if ((e.target as HTMLElement).closest(INTERACTIVE)) return;
  e.preventDefault();
  try {
    await getCurrentWindow().startDragging();
  } catch {
    // Non-fatal: worst case the window just doesn't move.
  }
}
