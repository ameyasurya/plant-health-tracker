import { useEffect, useRef, useState } from "react";
import type { AllPlantsRow, EventView } from "../types";

/*
 * Row tooltips carry only the care instruction -- the one part of the row
 * that actually gets clipped. The plant name and the "Water · Today" cue
 * are already legible on screen, so repeating them in a tooltip is noise.
 *
 * They're attached unconditionally rather than measured for overflow:
 * measuring needs a mount-time read that is wrong if it lands before web
 * fonts settle, which would hide the tooltip on exactly the clipped text
 * it exists to reveal.
 */

/** Relative-day wording. Derived here rather than reusing the backend's
 *  `cue` string so the urgency word can be styled on its own. */
function whenLabel(event: EventView): string {
  const d = event.days_until;
  if (d < -1) return `${-d} days overdue`;
  if (d === -1) return "1 day overdue";
  if (d === 0) return "Today";
  if (d === 1) return "Tomorrow";
  return `In ${d} days`;
}

function taskVerb(type: EventView["task_type"]): string {
  return type === "water" ? "Water" : "Feed";
}

interface ActionableProps {
  event: EventView;
  pendingLabel?: string | null;
  onDone: () => void;
  onSnooze: () => void;
  onSkip: () => void;
  onEdit: () => void;
  onUndo: () => void;
}

export function ActionableRow({ event, pendingLabel, onDone, onSnooze, onSkip, onEdit, onUndo }: ActionableProps) {
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

  if (pendingLabel) {
    return (
      <div className="row row-pending">
        <span className="row-pending-label ellipsis">{pendingLabel}</span>
        <button className="undo-btn" onClick={onUndo}>
          Undo
        </button>
      </div>
    );
  }

  const doneLabel = event.task_type === "water" ? "Mark as watered" : "Mark as fed";

  return (
    <div
      className={`row${event.bucket === "overdue" ? " row-overdue" : ""}`}
      title={event.instruction || undefined}
    >
      <TaskBadge type={event.task_type} />
      <div className="row-text">
        <div className="row-title ellipsis">{event.plant_name}</div>
        <div className="row-meta ellipsis">
          <span className={`row-when row-when-${event.bucket}`}>
            {taskVerb(event.task_type)} &middot; {whenLabel(event)}
          </span>
          {event.instruction && <span className="row-instruction"> &middot; {event.instruction}</span>}
        </div>
      </div>
      <button className="row-icon-btn row-done" aria-label={doneLabel} title={doneLabel} onClick={onDone}>
        <CheckIcon />
      </button>
      <div className="row-menu-wrap" ref={menuRef}>
        <button
          className="row-icon-btn"
          aria-label="More actions"
          title="More actions"
          aria-expanded={menuOpen}
          onClick={() => setMenuOpen((v) => !v)}
        >
          <DotsIcon />
        </button>
        {menuOpen && (
          <div className="row-menu">
            <button
              onClick={() => {
                setMenuOpen(false);
                onSnooze();
              }}
            >
              Snooze 1 day
            </button>
            <button
              onClick={() => {
                setMenuOpen(false);
                onSkip();
              }}
            >
              Skip, soil still wet
            </button>
            <button
              onClick={() => {
                setMenuOpen(false);
                onEdit();
              }}
            >
              Plant details
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export function AllPlantsListRow({ row, onEdit }: { row: AllPlantsRow; onEdit: () => void }) {
  return (
    <div
      className="row row-clickable"
      onClick={onEdit}
      role="button"
      tabIndex={0}
      // Only the scientific name is hidden here -- the common name is
      // already rendered, so a tooltip repeating it would be noise.
      title={row.scientific_name || undefined}
    >
      <div className="row-text">
        <div className="row-title ellipsis">
          {row.plant_name}
          {row.inferred && (
            <span className="inferred-dot" title="Some care details were inferred, so they are worth confirming">
              *
            </span>
          )}
        </div>
        <div className="row-meta ellipsis">
          <span className="row-stat" title={`Next water: ${row.next_water_label}`}>
            <DropletIcon /> {row.next_water_label}
          </span>
          <span className="row-stat" title={`Next feed: ${row.next_fertilize_label}`}>
            <FlaskIcon /> {row.next_fertilize_label}
          </span>
        </div>
      </div>
      <ChevronIcon />
    </div>
  );
}

/** Tinted badge: colour and glyph carry the water-vs-feed distinction, so
 *  the meta line doesn't have to compete for that job. */
function TaskBadge({ type }: { type: EventView["task_type"] }) {
  return (
    <span className={`task-badge task-badge-${type}`}>
      {type === "water" ? <DropletIcon /> : <FlaskIcon />}
    </span>
  );
}

const iconProps = {
  width: 14,
  height: 14,
  viewBox: "0 0 24 24",
  fill: "none",
  stroke: "currentColor",
  strokeWidth: 2,
  strokeLinecap: "round" as const,
  strokeLinejoin: "round" as const,
};

function DropletIcon() {
  return (
    <svg {...iconProps}>
      <path d="M12 2s6 7 6 11a6 6 0 1 1-12 0c0-4 6-11 6-11Z" />
    </svg>
  );
}

function FlaskIcon() {
  return (
    <svg {...iconProps}>
      <path d="M9 3h6M10 3v6l-5 9a1.5 1.5 0 0 0 1.3 2h11.4a1.5 1.5 0 0 0 1.3-2l-5-9V3" />
    </svg>
  );
}

function ChevronIcon() {
  return (
    <svg className="row-chevron" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M9 18l6-6-6-6" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 6 9 17l-5-5" />
    </svg>
  );
}

function DotsIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor" stroke="none">
      <circle cx="5" cy="12" r="1.8" />
      <circle cx="12" cy="12" r="1.8" />
      <circle cx="19" cy="12" r="1.8" />
    </svg>
  );
}
