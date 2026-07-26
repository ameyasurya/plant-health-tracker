import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { api } from "./api";
import { TitleBar } from "./components/TitleBar";
import { Tabs } from "./components/Tabs";
import { ActionableRow } from "./components/PlantRow";
import { OverviewList } from "./components/OverviewList";
import { TodoList } from "./components/TodoList";
import { Mascot, mascotStateForCounts } from "./components/Mascot";
import { EmptyState } from "./components/EmptyState";
import { AddPlantForm } from "./components/AddPlantForm";
import { EditPlantForm } from "./components/EditPlantForm";
import { PlantDetails } from "./components/PlantDetails";
import { SpacesPanel } from "./components/SpacesPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { ResizeGrip } from "./components/ResizeGrip";
import { WeatherStrip } from "./components/WeatherStrip";
import type {
  AllPlantsRow,
  EventView,
  NewPlant,
  PlantProfile,
  Settings,
  Space,
  Tab,
  TodoView,
  WeatherSummary,
} from "./types";

const appWindow = getCurrentWindow();

/** How long a Done/Snooze/Skip stays undo-able before it's actually sent to the backend. */
const CONFIRM_MS = 4000;

interface PendingAction {
  label: string;
  timeoutId: number;
}

type Overlay =
  | { kind: "none" }
  | { kind: "add-plant" }
  | { kind: "spaces" }
  | { kind: "settings" }
  | { kind: "details"; plant: PlantProfile }
  | { kind: "edit"; plant: PlantProfile };

export default function App() {
  const [tab, setTab] = useState<Tab>("today");
  const [due, setDue] = useState<EventView[]>([]);
  const [soon, setSoon] = useState<EventView[]>([]);
  const [all, setAll] = useState<AllPlantsRow[]>([]);
  const [spaces, setSpaces] = useState<Space[]>([]);
  const [todos, setTodos] = useState<TodoView[]>([]);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [weather, setWeather] = useState<WeatherSummary | null>(null);
  const [pending, setPending] = useState<Record<string, PendingAction>>({});
  const [overlay, setOverlay] = useState<Overlay>({ kind: "none" });
  const pendingRef = useRef(pending);
  pendingRef.current = pending;

  // Drives the shrink-to-fit density. Keyed off the widget's own measured
  // height rather than a CSS viewport media query: in the packaged app the
  // two are the same, but measuring the element also works inside the
  // browser preview harness, where the widget is a resizable div.
  const widgetRef = useRef<HTMLDivElement>(null);
  const [density, setDensity] = useState<"roomy" | "snug" | "tight">("roomy");

  useEffect(() => {
    const el = widgetRef.current;
    if (!el) return;
    const observer = new ResizeObserver(([entry]) => {
      const h = entry.contentRect.height;
      setDensity(h <= 320 ? "tight" : h <= 420 ? "snug" : "roomy");
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    return () => {
      Object.values(pendingRef.current).forEach((p) => window.clearTimeout(p.timeoutId));
    };
  }, []);

  // Startup pin state is applied in Rust (see lib.rs setup) so it doesn't
  // depend on the webview having loaded. This component only handles
  // runtime toggling.
  // To-dos are refreshed on their own as well as with everything else:
  // ticking a box shouldn't refetch the plant list and the forecast.
  const refreshTodos = useCallback(async () => {
    setTodos(await api.listTodos());
  }, []);

  const refresh = useCallback(async () => {
    const [dueRes, soonRes, allRes, spacesRes, settingsRes, weatherRes, todosRes] = await Promise.all([
      api.listDueToday(),
      api.listSoon(),
      api.listAllPlants(),
      api.listSpaces(),
      api.getSettings(),
      // Never let a missing/failed weather read break the whole refresh --
      // the plant list is the point, weather is decoration on top.
      api.getWeather().catch(() => null),
      api.listTodos(),
    ]);
    setDue(dueRes);
    setSoon(soonRes);
    setAll(allRes);
    setSpaces(spacesRes);
    setSettings(settingsRes);
    setWeather(weatherRes);
    setTodos(todosRes);
  }, []);

  useEffect(() => {
    refresh();
    // Refresh the forecast in the background on launch. It no-ops when
    // weather is off, unconfigured, or the cache is still fresh, and a
    // failure here must never surface as an error to the user.
    api
      .refreshWeather(false)
      .then((updated) => {
        if (updated) refresh();
      })
      .catch(() => {});

    const unlistenMarkAll = listen("mark-all-viewed", () => refresh());
    const unlistenSettings = listen("open-settings", () => setOverlay({ kind: "settings" }));
    // The tray has its own "Keep on top" toggle, which is the escape hatch
    // when an unpinned widget is behind another window. Reflect it here so
    // the pin button doesn't show a stale state.
    const unlistenPin = listen<boolean>("pin-changed", (e) => {
      setSettings((prev) => (prev ? { ...prev, pinned_on_top: e.payload } : prev));
    });
    return () => {
      unlistenMarkAll.then((f) => f());
      unlistenSettings.then((f) => f());
      unlistenPin.then((f) => f());
    };
  }, [refresh]);

  const overdueCount = due.filter((e) => e.bucket === "overdue").length;
  const dueTodayCount = due.filter((e) => e.bucket === "today").length;
  const mascotState = mascotStateForCounts(overdueCount, dueTodayCount, soon.length);

  function dropPending(p: Record<string, PendingAction>, id: string): Record<string, PendingAction> {
    const rest = { ...p };
    delete rest[id];
    return rest;
  }

  function scheduleAction(id: string, label: string, run: () => Promise<unknown>) {
    const timeoutId = window.setTimeout(() => {
      setPending((p) => dropPending(p, id));
      run().then(refresh);
    }, CONFIRM_MS);
    setPending((p) => ({ ...p, [id]: { label, timeoutId } }));
  }

  function handleUndo(id: string) {
    setPending((p) => {
      p[id] && window.clearTimeout(p[id].timeoutId);
      return dropPending(p, id);
    });
  }

  function handleDone(id: string, plantName: string) {
    scheduleAction(id, `Marked "${plantName}" done`, () => api.markDone(id));
  }
  function handleSnooze(id: string, plantName: string) {
    scheduleAction(id, `Snoozed "${plantName}" 1 day`, () => api.snooze(id, 1));
  }
  function handleSkip(id: string, plantName: string) {
    scheduleAction(id, `Skipped "${plantName}", soil still wet`, () => api.skipSoilWet(id));
  }

  async function handleShowDetails(plantId: string) {
    const plant = await api.getPlant(plantId);
    setOverlay({ kind: "details", plant });
  }

  async function handleAddPlant(plant: NewPlant) {
    await api.addPlant(plant);
    setOverlay({ kind: "none" });
    await refresh();
  }

  // Both of these write a single field rather than the whole Settings
  // object. Sending the full object from here meant a stale snapshot could
  // silently revert unrelated settings -- most visibly turning pin back
  // off, which then let the widget get buried behind other windows.
  async function handleSelectSpace(spaceId: string | null) {
    if (!settings) return;
    setSettings({ ...settings, active_space_id: spaceId });
    await api.setActiveSpace(spaceId);
    await refresh();
  }

  // The backend both persists this and applies it to the window, so the
  // widget and the tray can't end up disagreeing about the current layer.
  async function handleTogglePin() {
    if (!settings) return;
    const pinned = !settings.pinned_on_top;
    await api.setPinnedOnTop(pinned);
    setSettings({ ...settings, pinned_on_top: pinned });
  }

  async function handleMinimize() {
    await appWindow.hide();
  }

  const activeSpaceId = settings?.active_space_id ?? null;
  const defaultSpaceId = activeSpaceId ?? spaces[0]?.id ?? "balcony";

  // "All done for today" is only true if there was something to do. With no
  // plants in view -- a brand new install, or an empty space -- it claims
  // credit for work that doesn't exist and gives the user nothing to act
  // on, so say what's actually going on and point at the + button.
  const noPlantsHere = all.length === 0;
  const nothingHereText = activeSpaceId
    ? "No plants in this space yet. Press + to add one."
    : "No plants yet. Press + to add your first.";

  function renderRows(events: EventView[], emptyText: string) {
    if (events.length === 0) {
      return <EmptyState text={noPlantsHere ? nothingHereText : emptyText} plants={all} />;
    }
    return events.map((e) => (
      <ActionableRow
        key={e.id}
        event={e}
        pendingLabel={pending[e.id]?.label}
        onDone={() => handleDone(e.id, e.plant_name)}
        onSnooze={() => handleSnooze(e.id, e.plant_name)}
        onSkip={() => handleSkip(e.id, e.plant_name)}
        onEdit={() => handleShowDetails(e.plant_id)}
        onUndo={() => handleUndo(e.id)}
      />
    ));
  }

  return (
    <div className="widget" ref={widgetRef} data-density={density}>
      <div className="mascot-slot">
        <Mascot state={mascotState} />
      </div>
      <div className="card">
        <TitleBar
          pinned={settings?.pinned_on_top ?? false}
          spaces={spaces}
          activeSpaceId={activeSpaceId}
          onSelectSpace={handleSelectSpace}
          onManageSpaces={() => setOverlay({ kind: "spaces" })}
          onOpenSettings={() => setOverlay({ kind: "settings" })}
          onAddPlant={() => setOverlay({ kind: "add-plant" })}
          onTogglePin={handleTogglePin}
          onMinimize={handleMinimize}
        />
        <Tabs
          active={tab}
          dueCount={due.length}
          soonCount={soon.length}
          todoCount={todos.filter((t) => !t.done).length}
          onChange={setTab}
        />
        <div className="rows">
          {tab === "today" && renderRows(due, "All done for today.")}
          {tab === "soon" && renderRows(soon, "Nothing coming up in the next few days.")}
          {tab === "overview" &&
            (all.length === 0 ? (
              <EmptyState text={nothingHereText} plants={all} />
            ) : (
              <OverviewList rows={all} onSelect={handleShowDetails} />
            ))}
          {tab === "todo" && (
            <TodoList
              todos={todos}
              onAdd={async (text) => {
                await api.addTodo(text);
                await refreshTodos();
              }}
              onToggle={async (id) => {
                await api.toggleTodo(id);
                await refreshTodos();
              }}
              onDelete={async (id) => {
                await api.deleteTodo(id);
                await refreshTodos();
              }}
            />
          )}
        </div>

        {/* Footer, not a header: it explains the schedule rather than driving
            it, so it sits out of the way under the list instead of pushing
            the rows down. `.rows` is the only growing child, so this pins
            itself to the bottom of the card. */}
        <WeatherStrip weather={weather} />

        {overlay.kind === "add-plant" && (
          <AddPlantForm
            spaces={spaces}
            defaultSpaceId={defaultSpaceId}
            onCancel={() => setOverlay({ kind: "none" })}
            onSave={handleAddPlant}
          />
        )}
        {overlay.kind === "details" && (
          <PlantDetails
            plant={overlay.plant}
            spaces={spaces}
            onClose={() => setOverlay({ kind: "none" })}
            onEdit={() => setOverlay({ kind: "edit", plant: overlay.plant })}
            onLogCare={async (taskType, daysAgo) => {
              await api.logCare(overlay.plant.id, taskType, daysAgo);
              // Refresh behind the open panel so the lists are already
              // correct when the user closes it.
              await refresh();
            }}
          />
        )}
        {overlay.kind === "edit" && (
          <EditPlantForm
            plant={overlay.plant}
            spaces={spaces}
            onCancel={() => setOverlay({ kind: "details", plant: overlay.plant })}
            onSave={async (updated) => {
              await api.updatePlant(updated);
              setOverlay({ kind: "none" });
              await refresh();
            }}
            onDelete={async (plantId) => {
              await api.deletePlant(plantId);
              setOverlay({ kind: "none" });
              await refresh();
            }}
          />
        )}
        {overlay.kind === "spaces" && (
          <SpacesPanel
            spaces={spaces}
            onClose={() => setOverlay({ kind: "none" })}
            onAdd={async (name) => {
              await api.addSpace(name);
              await refresh();
            }}
            onRename={async (id, name) => {
              await api.renameSpace(id, name);
              await refresh();
            }}
            onDelete={async (id) => {
              await api.deleteSpace(id);
              await refresh();
            }}
          />
        )}

        {overlay.kind === "settings" && settings && (
          <SettingsPanel
            settings={settings}
            onClose={() => setOverlay({ kind: "none" })}
            onSave={async (next) => {
              await api.updateSettings(next);
              setSettings(next);
              // A changed location invalidates the cached forecast, so
              // force a refetch rather than waiting for the 6h window.
              await api.refreshWeather(true).catch(() => {});
              await refresh();
            }}
          />
        )}

        {/* Last child of .card so it paints over any open overlay panel. */}
        <ResizeGrip />
      </div>
    </div>
  );
}
