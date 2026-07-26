import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Space } from "../types";

/** Anything that should behave as a click rather than a drag handle. */
const INTERACTIVE = "button, input, select, textarea, a, [role='button']";

/** Unpinning sends the widget behind other windows, and the way back is a
 *  taskbar button on Windows but a Dock one on macOS. Naming the wrong one
 *  in the tooltip would send people looking in the wrong place. */
const RESTORE_TARGET = navigator.userAgent.includes("Mac") ? "Dock" : "taskbar";

/**
 * Drags the window from the title bar.
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
 */
async function startWindowDrag(e: React.PointerEvent) {
  if (e.button !== 0) return;
  if ((e.target as HTMLElement).closest(INTERACTIVE)) return;
  e.preventDefault();
  try {
    await getCurrentWindow().startDragging();
  } catch {
    // Non-fatal: worst case the window just doesn't move.
  }
}

interface Props {
  pinned: boolean;
  spaces: Space[];
  activeSpaceId: string | null;
  onSelectSpace: (spaceId: string | null) => void;
  onManageSpaces: () => void;
  onOpenSettings: () => void;
  onAddPlant: () => void;
  onTogglePin: () => void;
  onMinimize: () => void;
}

export function TitleBar({
  pinned,
  spaces,
  activeSpaceId,
  onSelectSpace,
  onManageSpaces,
  onOpenSettings,
  onAddPlant,
  onTogglePin,
  onMinimize,
}: Props) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menuOpen) return;
    function onOutside(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) setMenuOpen(false);
    }
    document.addEventListener("mousedown", onOutside);
    return () => document.removeEventListener("mousedown", onOutside);
  }, [menuOpen]);

  const activeName = activeSpaceId ? spaces.find((s) => s.id === activeSpaceId)?.name : null;

  return (
    <div
      onPointerDown={startWindowDrag}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "8px 10px",
        borderBottom: "0.5px solid var(--border, #e5e3dc)",
        flexShrink: 0,
        gap: 6,
      }}
    >
      <div className="space-switcher" ref={menuRef}>
        <button
          className="space-btn"
          title={`Showing: ${activeName ?? "All spaces"}. Click to switch`}
          aria-expanded={menuOpen}
          onClick={() => setMenuOpen((v) => !v)}
        >
          <span className="ellipsis">{activeName ?? "All spaces"}</span>
          <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5} strokeLinecap="round" strokeLinejoin="round">
            <path d="M6 9l6 6 6-6" />
          </svg>
        </button>
        {menuOpen && (
          <div className="row-menu space-menu">
            <button
              className={activeSpaceId === null ? "menu-active" : undefined}
              onClick={() => {
                setMenuOpen(false);
                onSelectSpace(null);
              }}
            >
              All spaces
            </button>
            {spaces.map((s) => (
              <button
                key={s.id}
                className={activeSpaceId === s.id ? "menu-active" : undefined}
                onClick={() => {
                  setMenuOpen(false);
                  onSelectSpace(s.id);
                }}
              >
                {s.name}
              </button>
            ))}
            <div className="menu-sep" />
            <button
              onClick={() => {
                setMenuOpen(false);
                onManageSpaces();
              }}
            >
              Manage spaces...
            </button>
            <button
              onClick={() => {
                setMenuOpen(false);
                onOpenSettings();
              }}
            >
              Settings...
            </button>
          </div>
        )}
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 2 }}>
        <button aria-label="Add plant" title="Add a plant" onClick={onAddPlant} className="icon-btn">
          <PlusIcon />
        </button>
        <button
          aria-label={pinned ? "Unpin from top" : "Pin on top"}
          title={
            pinned
              ? `Unpin so other windows can cover this. Click its ${RESTORE_TARGET} button to bring it back`
              : "Pin on top, keeping the widget above other windows"
          }
          onClick={onTogglePin}
          className={pinned ? "icon-btn active" : "icon-btn"}
        >
          <PinIcon />
        </button>
        <button
          aria-label="Minimize to tray"
          title="Hide to tray. Reopen from the tray icon"
          onClick={onMinimize}
          className="icon-btn"
        >
          <MinimizeIcon />
        </button>
      </div>
    </div>
  );
}

function PlusIcon() {
  return (
    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
      <path d="M12 5v14M5 12h14" />
    </svg>
  );
}

function PinIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
      <path d="M12 17v5M9 3h6l-1 6 3 3H7l3-3-1-6Z" />
    </svg>
  );
}

function MinimizeIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
      <path d="M5 12h14" />
    </svg>
  );
}
