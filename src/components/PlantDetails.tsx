import { useState } from "react";
import { Panel } from "./Panel";
import {
  FERTILIZE_OPTIONS,
  LIGHT_OPTIONS,
  MOISTURE_OPTIONS,
  type PlantProfile,
  type Space,
  type TaskType,
} from "../types";

function labelFor<T extends string>(options: { value: T; label: string }[], value: T): string {
  return options.find((o) => o.value === value)?.label ?? value;
}

/** Same vocabulary as the add-plant form, so "when did this happen" is asked
 *  the same way everywhere. There is no "not sure" here: the user is
 *  asserting they did it, only the day is fuzzy.
 *
 *  `phrase` exists because the button labels are clipped for width and do
 *  not read as English in a sentence ("watered this plant few days"). */
const WHEN_OPTIONS: { label: string; phrase: string; days: number }[] = [
  { label: "Today", phrase: "today", days: 0 },
  { label: "Yesterday", phrase: "yesterday", days: 1 },
  { label: "Few days", phrase: "a few days ago", days: 3 },
];

export function PlantDetails({
  plant,
  spaces,
  onClose,
  onEdit,
  onLogCare,
}: {
  plant: PlantProfile;
  spaces: Space[];
  onClose: () => void;
  onEdit: () => void;
  onLogCare: (taskType: TaskType, daysAgo: number) => Promise<void>;
}) {
  const spaceName = spaces.find((s) => s.id === plant.space_id)?.name;

  return (
    <Panel title={plant.common_name} onClose={onClose}>
      {plant.scientific_name && <div className="detail-sci">{plant.scientific_name}</div>}

      <div className="chip-row">
        {plant.category && <span className="chip">{plant.category}</span>}
        {spaceName && <span className="chip">{spaceName}</span>}
        {plant.is_hanging && <span className="chip">Hanging</span>}
      </div>

      <dl className="detail-grid">
        <dt>Light</dt>
        <dd>{labelFor(LIGHT_OPTIONS, plant.light)}</dd>
        <dt>Water</dt>
        <dd>{labelFor(MOISTURE_OPTIONS, plant.moisture_class)}</dd>
        <dt>Feed</dt>
        <dd>{labelFor(FERTILIZE_OPTIONS, plant.fertilize_group)}</dd>
      </dl>

      {plant.uses && (
        <section className="detail-section">
          <h4>Uses</h4>
          <p>{plant.uses}</p>
        </section>
      )}
      {plant.significance && (
        <section className="detail-section">
          <h4>Significance</h4>
          <p>{plant.significance}</p>
        </section>
      )}
      {plant.fun_fact && (
        <section className="detail-section">
          <h4>Did you know</h4>
          <p>{plant.fun_fact}</p>
        </section>
      )}
      {plant.notes && (
        <section className="detail-section">
          <h4>Notes</h4>
          <p>{plant.notes}</p>
        </section>
      )}

      {plant.inferred && (
        <div className="detail-inferred">
          Some care details for this plant were inferred, so they are worth confirming.
        </div>
      )}

      {/* Feeding cadences run 21 to 49 days but only reminders due within
          five days are listed, so without this there was no way to record a
          feed for most of the cycle. Watering gets the same control because
          two identical actions should not follow two different rules. */}
      <section className="detail-section">
        <h4>Already done it?</h4>
        <LogCareRow label="Watered" taskType="water" onLog={onLogCare} />
        <LogCareRow label="Fed" taskType="fertilize" onLog={onLogCare} />
      </section>

      <div className="panel-actions">
        <button onClick={onEdit}>Edit plant</button>
      </div>
    </Panel>
  );
}

function LogCareRow({
  label,
  taskType,
  onLog,
}: {
  label: string;
  taskType: TaskType;
  onLog: (taskType: TaskType, daysAgo: number) => Promise<void>;
}) {
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState<string | null>(null);

  async function log(days: number, phrase: string) {
    if (busy) return;
    setBusy(true);
    try {
      await onLog(taskType, days);
      // Confirm in place rather than closing the panel: rescheduling is
      // invisible from here, so without this the button appears to do nothing.
      setDone(phrase);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="logcare-row">
      <span className="logcare-label">{label}</span>
      {done ? (
        <span className="logcare-done">Logged {done}. Next one rescheduled.</span>
      ) : (
        <span className="logcare-opts">
          {WHEN_OPTIONS.map((o) => (
            <button
              key={o.label}
              type="button"
              className="lastdone-opt"
              disabled={busy}
              title={`Record that you ${label.toLowerCase()} this plant ${o.phrase}`}
              onClick={() => log(o.days, o.phrase)}
            >
              {o.label}
            </button>
          ))}
        </span>
      )}
    </div>
  );
}
