import { useEffect, useRef, useState } from "react";
import { Field, Panel } from "./Panel";
import { api } from "../api";
import {
  FERTILIZE_OPTIONS,
  LIGHT_OPTIONS,
  MOISTURE_OPTIONS,
  type CatalogEntry,
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

/**
 * Two-step add flow. Step one searches the bundled species catalog so the
 * care classes and knowledge copy come from real data rather than the user
 * guessing at "moisture class". Step two is the editable detail form,
 * pre-filled from the pick -- or blank if they skip the catalog entirely,
 * since no catalog can cover every plant.
 */
export function AddPlantForm({ spaces, defaultSpaceId, onCancel, onSave }: Props) {
  const [step, setStep] = useState<"search" | "details">("search");
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<CatalogEntry[]>([]);
  const [catalogId, setCatalogId] = useState<string | null>(null);

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

  // Debounced so typing doesn't fire a command per keystroke.
  const timerRef = useRef<number | undefined>(undefined);
  useEffect(() => {
    if (step !== "search") return;
    window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => {
      api.searchCatalog(query, 8).then(setResults).catch(() => setResults([]));
    }, 120);
    return () => window.clearTimeout(timerRef.current);
  }, [query, step]);

  function choose(entry: CatalogEntry) {
    setCatalogId(entry.id);
    setCommonName(entry.common_name);
    setScientificName(entry.scientific_name);
    setCategory(entry.category);
    setLight(entry.light);
    setMoisture(entry.moisture_class);
    setFertilize(entry.fertilize_group);
    setIsHanging(entry.typically_hanging);
    setStep("details");
  }

  function skipCatalog() {
    setCatalogId(null);
    setCommonName(query.trim());
    setStep("details");
  }

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
        catalog_id: catalogId,
      });
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  }

  if (step === "search") {
    return (
      <Panel title="Add a plant" onClose={onCancel}>
        {/* Sticky so the escape hatch stays reachable however far the
            results scroll -- on a short widget the panel's bottom actions
            are off-screen, which is exactly where it used to live. */}
        <div className="search-sticky">
          <span className="field-label">What plant is it?</span>
          <div className="search-row">
            <input
              autoFocus
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search: curry leaf, pothos..."
            />
            <button
              className="search-manual-btn"
              onClick={skipCatalog}
              title="Not in the catalog? Add it yourself and set the care details manually"
            >
              Add manually
            </button>
          </div>
        </div>

        <ul className="catalog-results">
          {results.map((entry) => (
            <li key={entry.id}>
              <button className="catalog-hit" onClick={() => choose(entry)}>
                <span className="catalog-name ellipsis">{entry.common_name}</span>
                <span className="catalog-sci ellipsis">{entry.scientific_name}</span>
              </button>
            </li>
          ))}
          {results.length === 0 && query.trim() && (
            <li className="catalog-empty">
              Nothing matches that. Use <strong>Add manually</strong> to enter it yourself.
            </li>
          )}
        </ul>

        <div className="panel-actions">
          <button onClick={onCancel}>Cancel</button>
        </div>
      </Panel>
    );
  }

  return (
    <Panel title="Add a plant" onClose={onCancel}>
      {catalogId && (
        <div className="catalog-badge">
          Care details filled in from the catalog. Adjust anything that doesn't match your plant.
        </div>
      )}

      <Field label="Name">
        <input autoFocus value={commonName} onChange={(e) => setCommonName(e.target.value)} />
      </Field>
      <Field label="Scientific name">
        <input value={scientificName} onChange={(e) => setScientificName(e.target.value)} />
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
        <button onClick={() => setStep("search")}>Back</button>
        <button className="btn-primary" disabled={!canSave} onClick={handleSave}>
          {saving ? "Adding..." : "Add plant"}
        </button>
      </div>
    </Panel>
  );
}
