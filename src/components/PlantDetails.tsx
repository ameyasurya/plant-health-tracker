import { Panel } from "./Panel";
import {
  FERTILIZE_OPTIONS,
  LIGHT_OPTIONS,
  MOISTURE_OPTIONS,
  type PlantProfile,
  type Space,
} from "../types";

function labelFor<T extends string>(options: { value: T; label: string }[], value: T): string {
  return options.find((o) => o.value === value)?.label ?? value;
}

export function PlantDetails({
  plant,
  spaces,
  onClose,
  onEdit,
}: {
  plant: PlantProfile;
  spaces: Space[];
  onClose: () => void;
  onEdit: () => void;
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

      <div className="panel-actions">
        <button onClick={onEdit}>Edit plant</button>
      </div>
    </Panel>
  );
}
