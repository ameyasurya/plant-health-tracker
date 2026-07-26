import { useEffect, useRef, useState } from "react";
import type { Space } from "../types";

interface Props {
  pinned: boolean;
  spaces: Space[];
  activeSpaceId: string | null;
  onSelectSpace: (spaceId: string | null) => void;
  onManageSpaces: () => void;
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
      // "deep" drags from anywhere in this subtree. A bare attribute only
      // fires when the click lands directly on this exact element, which
      // leaves nothing grabbable once the space switcher and buttons cover
      // the bar. Tauri's drag script still lets buttons/inputs through as
      // clicks, so the controls below keep working.
      data-tauri-drag-region="deep"
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
          title={`Showing: ${activeName ?? "All spaces"} — click to switch`}
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
              ? "Unpin — other windows can cover this; reopen from the tray icon if it gets buried"
              : "Pin on top — keep the widget above other windows"
          }
          onClick={onTogglePin}
          className={pinned ? "icon-btn active" : "icon-btn"}
        >
          <PinIcon />
        </button>
        <button
          aria-label="Minimize to tray"
          title="Hide to tray — reopen from the tray icon"
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
