import type { ReactNode } from "react";
import { startWindowDrag } from "../windowDrag";

/**
 * Full-card overlay used by the add-plant, details and spaces panels.
 *
 * The header doubles as a drag handle. This layer covers the card edge to
 * edge, including the real title bar, so without it the widget cannot be
 * moved at all while any panel is open.
 */
export function Panel({ title, onClose, children }: { title: string; onClose: () => void; children: ReactNode }) {
  return (
    <div className="panel">
      <div className="panel-head" onPointerDown={startWindowDrag}>
        <span className="panel-title ellipsis">{title}</span>
        <button className="row-icon-btn" aria-label="Close" title="Close" onClick={onClose}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round">
            <path d="M6 6l12 12M18 6L6 18" />
          </svg>
        </button>
      </div>
      <div className="panel-body">{children}</div>
    </div>
  );
}

export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="field">
      <span className="field-label">{label}</span>
      {children}
    </label>
  );
}
