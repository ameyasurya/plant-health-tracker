import { useState } from "react";
import { Field, Panel } from "./Panel";
import {
  FERTILIZE_OPTIONS,
  LIGHT_OPTIONS,
  MOISTURE_OPTIONS,
  type FertilizeGroup,
  type Light,
  type MoistureClass,
  type PlantProfile,
  type Space,
} from "../types";

interface Props {
  plant: PlantProfile;
  spaces: Space[];
  onCancel: () => void;
  onSave: (plant: PlantProfile) => Promise<void>;
  onDelete: (plantId: string) => Promise<void>;
}

/** Bounds on the watering nudge. Generous enough to cover a genuinely
 *  slow-drying pot, tight enough that a stray click can't park a plant
 *  a fortnight away by accident. */
const MIN_ADJUST = -3;
const MAX_ADJUST = 14;

/** Says what the number means, because "+2" on its own reads as a setting
 *  rather than a correction to a schedule the app already worked out. */
function describeAdjust(days: number): string {
  if (days === 0) return "As suggested";
  const magnitude = Math.abs(days);
  const unit = magnitude === 1 ? "day" : "days";
  return days > 0 ? `${magnitude} ${unit} later` : `${magnitude} ${unit} sooner`;
}

export function EditPlantForm({ plant, spaces, onCancel, onSave, onDelete }: Props) {
  const [commonName, setCommonName] = useState(plant.common_name);
  const [scientificName, setScientificName] = useState(plant.scientific_name);
  const [category, setCategory] = useState(plant.category);
  const [light, setLight] = useState<Light>(plant.light);
  const [moisture, setMoisture] = useState<MoistureClass>(plant.moisture_class);
  const [fertilize, setFertilize] = useState<FertilizeGroup>(plant.fertilize_group);
  const [isHanging, setIsHanging] = useState(plant.is_hanging);
  const [notes, setNotes] = useState(plant.notes);
  const [spaceId, setSpaceId] = useState(plant.space_id);
  const [waterAdjust, setWaterAdjust] = useState(plant.water_interval_adjust ?? 0);
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canSave = commonName.trim().length > 0 && !busy;

  async function run(fn: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(String(e));
      setBusy(false);
    }
  }

  function handleSave() {
    if (!canSave) return;
    // Spread the original so id and the catalog-sourced knowledge copy
    // survive the edit -- update_plant replaces the whole record.
    run(() =>
      onSave({
        ...plant,
        common_name: commonName.trim(),
        scientific_name: scientificName.trim(),
        category: category.trim(),
        light,
        moisture_class: moisture,
        fertilize_group: fertilize,
        is_hanging: isHanging,
        notes: notes.trim(),
        space_id: spaceId,
        water_interval_adjust: waterAdjust,
      }),
    );
  }

  return (
    <Panel title={`Edit ${plant.common_name}`} onClose={onCancel}>
      <Field label="Name">
        <input autoFocus value={commonName} onChange={(e) => setCommonName(e.target.value)} />
      </Field>
      <Field label="Scientific name">
        <input value={scientificName} onChange={(e) => setScientificName(e.target.value)} />
      </Field>
      <Field label="Category">
        <input value={category} onChange={(e) => setCategory(e.target.value)} />
      </Field>
      <Field label="Space">
        <select value={spaceId} onChange={(e) => setSpaceId(e.target.value)}>
          {spaces.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
      </Field>
      <Field label="Light">
        <select value={light} onChange={(e) => setLight(e.target.value as Light)}>
          {LIGHT_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </Field>
      <Field label="Watering">
        <select value={moisture} onChange={(e) => setMoisture(e.target.value as MoistureClass)}>
          {MOISTURE_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </Field>
      <Field label="Feeding">
        <select value={fertilize} onChange={(e) => setFertilize(e.target.value as FertilizeGroup)}>
          {FERTILIZE_OPTIONS.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
      </Field>
      <label className="checkbox-row">
        <input type="checkbox" checked={isHanging} onChange={(e) => setIsHanging(e.target.checked)} />
        <span>Hanging pot (dries out faster)</span>
      </label>

      {/* The species tables can't know about pot size, soil mix or exactly
          where this plant sits, all of which change how fast it dries. This
          shifts the schedule rather than replacing it, so seasonal and
          weather adjustments still apply on top. */}
      <Field label="Watering pace">
        <div className="pace-row">
          <button
            type="button"
            className="pace-step"
            aria-label="Water more often"
            title="Water more often"
            disabled={waterAdjust <= MIN_ADJUST}
            onClick={() => setWaterAdjust((v) => Math.max(MIN_ADJUST, v - 1))}
          >
            &minus;
          </button>
          <span className="pace-value">{describeAdjust(waterAdjust)}</span>
          <button
            type="button"
            className="pace-step"
            aria-label="Water less often"
            title="Water less often"
            disabled={waterAdjust >= MAX_ADJUST}
            onClick={() => setWaterAdjust((v) => Math.min(MAX_ADJUST, v + 1))}
          >
            +
          </button>
        </div>
      </Field>

      <Field label="Notes">
        <textarea rows={2} value={notes} onChange={(e) => setNotes(e.target.value)} />
      </Field>

      {error && <div className="panel-error">{error}</div>}

      <div className="panel-actions">
        <button onClick={onCancel}>Cancel</button>
        <button className="btn-primary" disabled={!canSave} onClick={handleSave}>
          {busy ? "Saving..." : "Save"}
        </button>
      </div>

      {/* Deleting also discards this plant's care history, so it sits apart
          from the save actions and asks a second time. */}
      <div className="danger-zone">
        {confirmingDelete ? (
          <>
            <span className="danger-text">
              Delete {plant.common_name} and its care history? This can't be undone.
            </span>
            <div className="danger-actions">
              <button onClick={() => setConfirmingDelete(false)}>Keep</button>
              <button className="btn-danger" disabled={busy} onClick={() => run(() => onDelete(plant.id))}>
                Delete
              </button>
            </div>
          </>
        ) : (
          <button className="danger-link" onClick={() => setConfirmingDelete(true)}>
            Delete this plant
          </button>
        )}
      </div>
    </Panel>
  );
}
