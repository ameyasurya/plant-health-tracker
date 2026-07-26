import { useState } from "react";
import type { TodoView } from "../types";

/**
 * A plain checklist, sitting alongside the plant tabs.
 *
 * Nothing is ever cleared automatically. An item left unticked from an
 * earlier day stays put and is labelled with its age, because quietly
 * deleting something the user typed is a worse failure than a list that
 * needs tidying by hand.
 *
 * The add field is pinned above the scrolling list for the same reason the
 * add-plant search moved: at the two-row height this widget supports, a
 * control at the bottom is a control nobody finds.
 */
export function TodoList({
  todos,
  onAdd,
  onToggle,
  onDelete,
}: {
  todos: TodoView[];
  onAdd: (text: string) => void;
  onToggle: (id: string) => void;
  onDelete: (id: string) => void;
}) {
  const [draft, setDraft] = useState("");

  function submit(e: React.FormEvent) {
    e.preventDefault();
    const text = draft.trim();
    if (!text) return;
    onAdd(text);
    setDraft("");
  }

  const openCount = todos.filter((t) => !t.done).length;

  return (
    <div className="todo-wrap">
      <form className="todo-add" onSubmit={submit}>
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          placeholder="Add a to-do"
          aria-label="Add a to-do"
          maxLength={200}
        />
        <button type="submit" className="icon-btn" aria-label="Add to-do" title="Add to-do" disabled={!draft.trim()}>
          <PlusIcon />
        </button>
      </form>

      {todos.length === 0 ? (
        <div className="empty-state">
          <div className="empty-headline">Nothing on the list.</div>
          <div>Add anything you want to remember while the widget is open.</div>
        </div>
      ) : (
        <div className="todo-items">
          {todos.map((t) => (
            <div key={t.id} className={t.done ? "todo-item todo-done" : "todo-item"}>
              <button
                className="todo-check"
                role="checkbox"
                aria-checked={t.done}
                aria-label={t.done ? `Mark "${t.text}" as not done` : `Mark "${t.text}" as done`}
                title={t.done ? "Mark as not done" : "Mark as done"}
                onClick={() => onToggle(t.id)}
              >
                {t.done && <TickIcon />}
              </button>
              <span className="todo-text">
                {t.text}
                {t.carried_over && <span className="todo-age">{t.age_label}</span>}
              </span>
              <button
                className="todo-delete"
                aria-label={`Delete "${t.text}"`}
                title="Delete"
                onClick={() => onDelete(t.id)}
              >
                <CrossIcon />
              </button>
            </div>
          ))}
        </div>
      )}

      {todos.length > 0 && (
        <div className="todo-summary">
          {openCount === 0 ? "All done." : `${openCount} left`}
        </div>
      )}
    </div>
  );
}

function PlusIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden="true">
      <path d="M12 5v14M5 12h14" />
    </svg>
  );
}

function TickIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M20 6 9 17l-5-5" />
    </svg>
  );
}

function CrossIcon() {
  return (
    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" aria-hidden="true">
      <path d="M18 6 6 18M6 6l12 12" />
    </svg>
  );
}
