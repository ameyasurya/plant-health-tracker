import { useState } from "react";
import { Field, Panel } from "./Panel";
import {
  FERTILIZE_OPTIONS,
  LIGHT_OPTIONS,
  MOISTURE_OPTIONS,
  type FertilizeGroup,
  type Light,
  type MoistureClass,
  type NewPlant,
  type Space,
} from "../types";

interface Props {
  spaces: Space[];
  defaultSpaceId: string;
  onCancel: () => void;
  onSave: (plant: NewPlant) => Promise<void>;
}

export function AddPlantForm({ spaces, defaultSpaceId, onCancel, onSave }: Props) {
  const [commonName, setCommonName] = useState("");
  const [scientificName, setScientificName] = useState("");
  const [category, setCategory] = useState("");
  const [light, setLight] = useState<Light>("bright_light");
  const [moisture, setMoisture] = useState<MoistureClass>("moderate");
  const [fertilize, setFertilize] = useState<FertilizeGroup>("foliage");
  const [isHanging, setIsHanging] = useState(false);
  const [notes, setNotes] = useState("");
  const [spaceId, setSpaceId] = useState(defaultSpaceId);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canSave = commonName.trim().length > 0 && !saving;

  async function handleSave() {
    if (!canSave) return;
    setSaving(true);
    setError(null);
    try {
      await onSave({
        common_name: commonName,
        scientific_name: scientificName,
        category,
        light,
        moisture_class: moisture,
        fertilize_group: fertilize,
        is_hanging: isHanging,
        notes,
        space_id: spaceId,
      });
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  }

  return (
    <Panel title="Add a plant" onClose={onCancel}>
      <Field label="Name">
        <input
          autoFocus
          value={commonName}
          onChange={(e) => setCommonName(e.target.value)}
          placeholder="Curry Leaf"
        />
      </Field>
      <Field label="Scientific name">
        <input
          value={scientificName}
          onChange={(e) => setScientificName(e.target.value)}
          placeholder="Murraya koenigii"
        />
      </Field>
      <Field label="Category">
        <input value={category} onChange={(e) => setCategory(e.target.value)} placeholder="Herb" />
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
      <Field label="Notes">
        <textarea rows={2} value={notes} onChange={(e) => setNotes(e.target.value)} />
      </Field>

      {error && <div className="panel-error">{error}</div>}

      <div className="panel-actions">
        <button onClick={onCancel}>Cancel</button>
        <button className="btn-primary" disabled={!canSave} onClick={handleSave}>
          {saving ? "Adding..." : "Add plant"}
        </button>
      </div>
    </Panel>
  );
}
