import { useState } from "react";
import { Panel } from "./Panel";
import type { Space } from "../types";

interface Props {
  spaces: Space[];
  onClose: () => void;
  onAdd: (name: string) => Promise<void>;
  onRename: (spaceId: string, name: string) => Promise<void>;
  onDelete: (spaceId: string) => Promise<void>;
}

export function SpacesPanel({ spaces, onClose, onAdd, onRename, onDelete }: Props) {
  const [newName, setNewName] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function run(fn: () => Promise<void>) {
    setBusy(true);
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError(String(e));
    }
    setBusy(false);
  }

  return (
    <Panel title="Spaces" onClose={onClose}>
      <p className="panel-hint">Group plants by where they live: balcony, kitchen, bedroom.</p>

      <ul className="space-list">
        {spaces.map((s) => (
          <li key={s.id}>
            {editingId === s.id ? (
              <>
                <input
                  autoFocus
                  value={editingName}
                  onChange={(e) => setEditingName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      run(async () => {
                        await onRename(s.id, editingName);
                        setEditingId(null);
                      });
                    }
                    if (e.key === "Escape") setEditingId(null);
                  }}
                />
                <button
                  disabled={busy}
                  onClick={() =>
                    run(async () => {
                      await onRename(s.id, editingName);
                      setEditingId(null);
                    })
                  }
                >
                  Save
                </button>
              </>
            ) : (
              <>
                <span className="ellipsis space-name" title={s.name}>
                  {s.name}
                </span>
                <button
                  title={`Rename "${s.name}"`}
                  onClick={() => {
                    setEditingId(s.id);
                    setEditingName(s.name);
                    setError(null);
                  }}
                >
                  Rename
                </button>
                <button
                  disabled={busy || spaces.length <= 1}
                  title={
                    spaces.length <= 1
                      ? "You need at least one space"
                      : `Delete "${s.name}". Its plants move to another space, nothing is lost`
                  }
                  onClick={() => run(() => onDelete(s.id))}
                >
                  Delete
                </button>
              </>
            )}
          </li>
        ))}
      </ul>

      <div className="space-add">
        <input
          value={newName}
          placeholder="New space name"
          onChange={(e) => setNewName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && newName.trim()) {
              run(async () => {
                await onAdd(newName);
                setNewName("");
              });
            }
          }}
        />
        <button
          className="btn-primary"
          disabled={busy || !newName.trim()}
          onClick={() =>
            run(async () => {
              await onAdd(newName);
              setNewName("");
            })
          }
        >
          Add
        </button>
      </div>

      {error && <div className="panel-error">{error}</div>}
    </Panel>
  );
}
