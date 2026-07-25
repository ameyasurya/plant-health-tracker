import { useMemo, useState } from "react";
import type { AllPlantsRow } from "../types";

/**
 * All-clear state. Beyond confirming there's nothing to do, it surfaces a
 * fact about a plant the user actually owns -- the widget is on screen all
 * day, so this is the one moment it has something to offer besides chores.
 */
export function EmptyState({ text, plants }: { text: string; plants: AllPlantsRow[] }) {
  const withFacts = useMemo(() => plants.filter((p) => p.fun_fact.trim().length > 0), [plants]);
  const [index, setIndex] = useState(() => Math.floor(Math.random() * 1000));

  if (withFacts.length === 0) {
    return <div className="empty-state">{text}</div>;
  }

  const plant = withFacts[index % withFacts.length];

  return (
    <div className="empty-state">
      <div className="empty-headline">{text}</div>
      <div className="fact-card">
        <div className="fact-head">
          <span className="fact-kicker">Did you know</span>
          {withFacts.length > 1 && (
            <button
              className="fact-next"
              aria-label="Another fact"
              title="Show another fact"
              onClick={() => setIndex((i) => i + 1)}
            >
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round">
                <path d="M21 12a9 9 0 1 1-3-6.7" />
                <path d="M21 4v5h-5" />
              </svg>
            </button>
          )}
        </div>
        <p className="fact-text" title={plant.fun_fact}>
          {plant.fun_fact}
        </p>
        <div className="fact-source">{plant.plant_name}</div>
      </div>
    </div>
  );
}
