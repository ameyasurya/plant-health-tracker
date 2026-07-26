import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Visible resize affordance for the bottom-right corner.
 *
 * A frameless window still has invisible resize borders, but nothing on
 * screen says so -- users reasonably assume a chrome-less widget is a
 * fixed size. This draws the usual diagonal grip and drives the resize
 * explicitly via startResizeDragging, so the hit area is a real target
 * rather than a few pixels of guesswork at the window edge.
 */
export function ResizeGrip() {
  async function begin(e: React.PointerEvent) {
    // Left button only, and swallow the event so it can't also start a
    // window drag or land on whatever is underneath.
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    try {
      await getCurrentWindow().startResizeDragging("SouthEast");
    } catch {
      // Non-fatal: worst case the invisible window border still works.
    }
  }

  return (
    <div
      className="resize-grip"
      onPointerDown={begin}
      role="separator"
      aria-label="Resize widget"
      title="Drag to resize"
    >
      <svg width="12" height="12" viewBox="0 0 12 12" aria-hidden="true">
        <path d="M11 4 L4 11 M11 8 L8 11" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none" />
      </svg>
    </div>
  );
}
