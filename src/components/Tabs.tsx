import type { Tab } from "../types";

interface Props {
  active: Tab;
  dueCount: number;
  soonCount: number;
  onChange: (tab: Tab) => void;
}

export function Tabs({ active, dueCount, soonCount, onChange }: Props) {
  return (
    <div style={{ display: "flex", gap: 2, padding: "8px 10px 6px", flexShrink: 0 }}>
      <TabButton label="Due today" count={dueCount} countColor="var(--text-danger, #a32d2d)" isActive={active === "today"} onClick={() => onChange("today")} />
      <TabButton label="Soon" count={soonCount} countColor="var(--text-muted, #888780)" isActive={active === "soon"} onClick={() => onChange("soon")} />
      <TabButton label="All plants" isActive={active === "all"} onClick={() => onChange("all")} />
    </div>
  );
}

function TabButton({
  label,
  count,
  countColor,
  isActive,
  onClick,
}: {
  label: string;
  count?: number;
  countColor?: string;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      style={{
        fontSize: 11.5,
        padding: "4px 9px",
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
