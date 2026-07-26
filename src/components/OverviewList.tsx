import type { AllPlantsRow, Bucket } from "../types";

/**
 * Every plant grouped under the space it lives in, with its water and feed
 * state at a glance.
 *
 * This replaces the old flat "All plants" list. That list answered "what do
 * I own", which the Today and Soon tabs already imply; grouping by space and
 * showing both statuses answers "is anything being neglected", which they
 * don't.
 *
 * Grouping follows the active space filter rather than overriding it: with a
 * space selected you get one group, with "All spaces" you get them all.
 */
export function OverviewList({
  rows,
  onSelect,
}: {
  rows: AllPlantsRow[];
  onSelect: (plantId: string) => void;
}) {
  // Preserve the backend's alphabetical plant order within each space, and
  // order the groups by name so the list doesn't reshuffle between refreshes.
  const groups = new Map<string, AllPlantsRow[]>();
  for (const row of rows) {
    const existing = groups.get(row.space_name);
    if (existing) existing.push(row);
    else groups.set(row.space_name, [row]);
  }
  const ordered = [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));

  return (
    <>
      {ordered.map(([spaceName, plants]) => (
        <div key={spaceName} className="overview-group">
          <div className="overview-space">
            <span className="ellipsis">{spaceName}</span>
            <span className="overview-count">{plants.length}</span>
          </div>
          {plants.map((row) => (
            <button
              key={row.plant_id}
              className="overview-row"
              // Tooltip carries the scientific name, the one thing not
              // already on screen. The label is set explicitly because the
              // chips' own tooltips would otherwise be concatenated into a
              // confusing accessible name for the row.
              title={row.scientific_name || undefined}
              aria-label={`${row.plant_name}: water ${row.next_water_label}, feed ${row.next_fertilize_label}`}
              onClick={() => onSelect(row.plant_id)}
            >
              <span className="overview-name ellipsis">
                {row.plant_name}
                {row.inferred && (
                  <span className="inferred-dot" title="Some care details were inferred, so they are worth confirming">
                    *
                  </span>
                )}
              </span>
              <span className="overview-chips">
                <StatusChip kind="water" status={row.water_status} label={row.next_water_label} />
                <StatusChip kind="feed" status={row.fertilize_status} label={row.next_fertilize_label} />
              </span>
            </button>
          ))}
        </div>
      ))}
    </>
  );
}

/**
 * One task's state. The word is spelled out rather than shown as a bare
 * colour, because colour alone would leave "water" and "feed" impossible to
 * tell apart for anyone who can't distinguish them.
 */
function StatusChip({ kind, status, label }: { kind: "water" | "feed"; status: Bucket; label: string }) {
  const verb = kind === "water" ? "Water" : "Feed";
  return (
    <span className={`status-chip status-${status} chip-${kind}`} title={`${verb} ${label}`}>
      {kind === "water" ? <DropIcon /> : <FeedIcon />}
      <span>{label}</span>
    </span>
  );
}

function DropIcon() {
  return (
    <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.2} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M12 3s6 6.6 6 10.5a6 6 0 0 1-12 0C6 9.6 12 3 12 3Z" />
    </svg>
  );
}

function FeedIcon() {
  return (
    <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.2} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M12 20V9M12 9C12 6 14 4 17 4c0 3-2 5-5 5ZM12 12C12 9.5 9.5 7.5 7 7.5c0 2.5 2.5 4.5 5 4.5Z" />
    </svg>
  );
}
