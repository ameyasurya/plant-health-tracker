import type { Tab } from "../types";

interface Props {
  active: Tab;
  dueCount: number;
  soonCount: number;
  todoCount: number;
  onChange: (tab: Tab) => void;
}

/**
 * Four tabs in a 380px window (340px at its narrowest), so the labels are
 * deliberately short: "Due today" and "All plants" no longer fit alongside
 * two more. Counts stay on Today and Soon, where urgency matters, plus the
 * open to-do count since that tab is otherwise invisible state.
 */
export function Tabs({ active, dueCount, soonCount, todoCount, onChange }: Props) {
  return (
    <div style={{ display: "flex", gap: 2, padding: "8px 10px 6px", flexShrink: 0 }}>
      <TabButton label="Today" title="Due today" count={dueCount} countColor="var(--text-danger, #a32d2d)" isActive={active === "today"} onClick={() => onChange("today")} />
      <TabButton label="Soon" title="Coming up in the next few days" count={soonCount} countColor="var(--text-muted, #888780)" isActive={active === "soon"} onClick={() => onChange("soon")} />
      <TabButton label="Plants" title="Every plant, grouped by space" isActive={active === "overview"} onClick={() => onChange("overview")} />
      <TabButton label="To-do" title="Your checklist" count={todoCount} countColor="var(--text-muted, #888780)" isActive={active === "todo"} onClick={() => onChange("todo")} />
    </div>
  );
}

function TabButton({
  label,
  title,
  count,
  countColor,
  isActive,
  onClick,
}: {
  label: string;
  title?: string;
  count?: number;
  countColor?: string;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      title={title}
      style={{
        fontSize: 11.5,
        padding: "4px 7px",
        background: isActive ? undefined : "transparent",
        borderColor: isActive ? undefined : "transparent",
      }}
    >
      {label}
      {typeof count === "number" && count > 0 && (
        <>
          {" "}
          <span style={{ color: countColor }}>{count}</span>
        </>
      )}
    </button>
  );
}
