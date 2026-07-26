// Browser-only preview harness. Mocks the Tauri IPC bridge with
// @tauri-apps/api's official mocks module so the real App can be opened
// and iterated on in a normal browser tab (screenshots, devtools, hot
// reload) instead of the native window. Never imported by index.html /
// main.tsx, and vite.config.ts doesn't list preview.html as a build
// input, so none of this ships in the packaged app.
import React from "react";
import ReactDOM from "react-dom/client";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import type { AllPlantsRow, EventView, NewPlant, PlantProfile, Settings, Space } from "./types";
import "./styles.css";

mockWindows("main");

type TaskType = "water" | "fertilize";

interface MockPlant {
  id: string;
  name: string;
  scientificName: string;
  inferred?: boolean;
  spaceId?: string;
  funFact?: string;
}

interface MockEvent {
  id: string;
  plantId: string;
  taskType: TaskType;
  status: "pending" | "done" | "snoozed" | "skipped";
  dueOffset: number;
}

const PLANTS: MockPlant[] = [
  { id: "p1", name: "Money Plant", scientificName: "Epipremnum aureum" },
  { id: "p2", name: "Tulsi", scientificName: "Ocimum tenuiflorum" },
  { id: "p3", name: "Curry Leaf", scientificName: "Murraya koenigii" },
  { id: "p4", name: "Ixora", scientificName: "Ixora coccinea", inferred: true },
  { id: "p5", name: "Hibiscus", scientificName: "Hibiscus rosa-sinensis" },
  { id: "p6", name: "Aloe Vera", scientificName: "Aloe barbadensis" },
  { id: "p7", name: "Snake Plant", scientificName: "Dracaena trifasciata" },
  { id: "p8", name: "Marigold", scientificName: "Tagetes erecta" },
  { id: "p9", name: "Jasmine", scientificName: "Jasminum sambac" },
  { id: "p10", name: "Areca Palm", scientificName: "Dypsis lutescens" },
  { id: "p11", name: "Indian Borage", scientificName: "Plectranthus amboinicus", inferred: true },
  { id: "p12", name: "Rose", scientificName: "Rosa hybrid" },
  { id: "p13", name: "Mint", scientificName: "Mentha spicata" },
  { id: "p14", name: "Peace Lily", scientificName: "Spathiphyllum wallisii" },
  { id: "p15", name: "Bougainvillea", scientificName: "Bougainvillea glabra" },
  { id: "p16", name: "Spider Plant", scientificName: "Chlorophytum comosum" },
  { id: "p17", name: "Coleus", scientificName: "Coleus scutellarioides" },
  { id: "p18", name: "Lemongrass", scientificName: "Cymbopogon citratus" },
  { id: "p19", name: "Chili Pepper", scientificName: "Capsicum annuum" },
];

let events: MockEvent[] = PLANTS.flatMap((p, i) => [
  { id: `${p.id}-w`, plantId: p.id, taskType: "water" as TaskType, status: "pending" as const, dueOffset: ((i * 3 - 4) % 12) },
  { id: `${p.id}-f`, plantId: p.id, taskType: "fertilize" as TaskType, status: "pending" as const, dueOffset: ((i * 5 + 2) % 20) },
]);

let settings: Settings = {
  notification_time: "08:00",
  launch_at_startup: true,
  density_mode: "compact",
  pinned_on_top: false,
  last_digest_sent_on: null,
  active_space_id: null,
  location: {
    label: "Bengaluru, Karnataka, India",
    latitude: 12.97,
    longitude: 77.59,
    timezone: "Asia/Kolkata",
    country_code: "IN",
  },
  weather_enabled: true,
};

let spaces: Space[] = [
  { id: "balcony", name: "Balcony" },
  { id: "kitchen", name: "Kitchen" },
];

const FACTS: Record<string, string> = {
  p1: "In the wild it climbs trees and its leaves grow enormous; indoors they stay small because it never gets to climb.",
  p3: "It belongs to the citrus family -- crush a leaf and you can catch a faint citrus note beneath the spice.",
  p4: "Each ball of colour is a cluster of many slender, four-petalled tubular flowers.",
  p6: "Its thick leaves store water like a succulent, so overwatering harms it faster than forgetting to water.",
  p9: "Many jasmines release their scent most strongly after dark, timed to night-flying pollinators.",
};

function dayLabel(daysUntil: number): string {
  if (daysUntil < 0) return "overdue";
  if (daysUntil === 0) return "today";
  if (daysUntil === 1) return "in 1 day";
  return `in ${daysUntil} days`;
}

function bucketFor(daysUntil: number): "overdue" | "today" | "soon" {
  if (daysUntil < 0) return "overdue";
  if (daysUntil === 0) return "today";
  return "soon";
}

function cueLabel(taskType: TaskType, daysUntil: number): string {
  const verb = taskType === "water" ? "Water" : "Feed";
  return `${verb} · ${dayLabel(daysUntil)}`;
}

function instructionFor(taskType: TaskType, plant: MockPlant): string {
  if (taskType === "water") return `Check topsoil for ${plant.name.toLowerCase()}; water if dry 2-3cm down.`;
  return `Feed ${plant.name.toLowerCase()} with balanced fertilizer.`;
}

function buildEventView(event: MockEvent, plant: MockPlant): EventView {
  return {
    id: event.id,
    plant_id: plant.id,
    plant_name: plant.name,
    task_type: event.taskType,
    due_at: new Date().toISOString().slice(0, 10),
    days_until: event.dueOffset,
    bucket: bucketFor(event.dueOffset),
    cue: cueLabel(event.taskType, event.dueOffset),
    instruction: instructionFor(event.taskType, plant),
  };
}

function isLive(e: MockEvent) {
  return e.status === "pending" || e.status === "snoozed";
}

function listDueAndSoon(): { due: EventView[]; soon: EventView[] } {
  const due: EventView[] = [];
  const soon: EventView[] = [];
  for (const event of events) {
    if (!isLive(event)) continue;
    const plant = PLANTS.find((p) => p.id === event.plantId);
    if (!plant) continue;
    const view = buildEventView(event, plant);
    if (view.bucket === "overdue" || view.bucket === "today") due.push(view);
    else if (event.dueOffset <= 5) soon.push(view);
  }
  due.sort((a, b) => a.days_until - b.days_until);
  soon.sort((a, b) => a.days_until - b.days_until);
  return { due, soon };
}

function listAllPlants(): AllPlantsRow[] {
  const rows = PLANTS.map((plant) => {
    const waterEvents = events.filter((e) => e.plantId === plant.id && e.taskType === "water" && isLive(e));
    const fertEvents = events.filter((e) => e.plantId === plant.id && e.taskType === "fertilize" && isLive(e));
    const water = waterEvents.length ? Math.min(...waterEvents.map((e) => e.dueOffset)) : 0;
    const fert = fertEvents.length ? Math.min(...fertEvents.map((e) => e.dueOffset)) : 0;
    return {
      plant_id: plant.id,
      plant_name: plant.name,
      scientific_name: plant.scientificName,
      next_water: "",
      next_water_label: dayLabel(water),
      next_fertilize: "",
      next_fertilize_label: dayLabel(fert),
      inferred: !!plant.inferred,
      fun_fact: FACTS[plant.id] ?? "",
    };
  });
  rows.sort((a, b) => a.plant_name.localeCompare(b.plant_name));
  return rows;
}

mockIPC((cmd, payload) => {
  const args = (payload ?? {}) as Record<string, unknown>;
  switch (cmd) {
    case "list_due_today":
      return listDueAndSoon().due;
    case "list_soon":
      return listDueAndSoon().soon;
    case "list_all_plants":
      return listAllPlants();
    case "mark_done": {
      const id = args.eventId as string;
      const idx = events.findIndex((e) => e.id === id);
      if (idx >= 0) {
        events[idx] = { ...events[idx], status: "done" };
        const src = events[idx];
        events.push({ id: `${id}-next`, plantId: src.plantId, taskType: src.taskType, status: "pending", dueOffset: 7 });
      }
      return undefined;
    }
    case "snooze": {
      const id = args.eventId as string;
      const days = (args.days as number) ?? 1;
      events = events.map((e) => (e.id === id ? { ...e, status: "snoozed", dueOffset: days } : e));
      return undefined;
    }
    case "skip_soil_wet": {
      const id = args.eventId as string;
      const idx = events.findIndex((e) => e.id === id);
      if (idx >= 0) {
        events[idx] = { ...events[idx], status: "skipped" };
        const src = events[idx];
        events.push({ id: `${id}-recheck`, plantId: src.plantId, taskType: src.taskType, status: "pending", dueOffset: 2 });
      }
      return undefined;
    }
    case "get_settings":
      return settings;
    case "update_settings":
      settings = args.settings as Settings;
      return undefined;
    case "list_spaces":
      return spaces;
    case "add_space": {
      const name = (args.name as string).trim();
      if (spaces.some((s) => s.name.toLowerCase() === name.toLowerCase())) {
        throw new Error(`a space called "${name}" already exists`);
      }
      spaces = [...spaces, { id: `sp-${Date.now()}`, name }];
      return spaces[spaces.length - 1];
    }
    case "rename_space":
      spaces = spaces.map((s) => (s.id === args.spaceId ? { ...s, name: args.name as string } : s));
      return undefined;
    case "delete_space": {
      if (spaces.length <= 1) throw new Error("can't delete the only space");
      spaces = spaces.filter((s) => s.id !== args.spaceId);
      return undefined;
    }
    case "get_plant": {
      const plant = PLANTS.find((p) => p.id === args.plantId);
      if (!plant) throw new Error("plant not found");
      const profile: PlantProfile = {
        id: plant.id,
        common_name: plant.name,
        scientific_name: plant.scientificName,
        category: "Flowering",
        light: "bright_light",
        moisture_class: "moderate",
        fertilize_group: "foliage",
        is_hanging: false,
        notes: "Preview mock plant.",
        inferred: !!plant.inferred,
        space_id: plant.spaceId ?? "balcony",
        uses: "Ornamental foliage and a bit of greenery for a small balcony.",
        significance: "Stand-in text so the details panel can be styled without the Rust backend.",
        fun_fact: FACTS[plant.id] ?? "This one has no fun fact recorded yet.",
      };
      return profile;
    }
    case "add_plant": {
      const p = args.plant as NewPlant;
      const id = `new-${Date.now()}`;
      PLANTS.push({ id, name: p.common_name, scientificName: p.scientific_name, spaceId: p.space_id });
      events.push({ id: `${id}-w`, plantId: id, taskType: "water", status: "pending", dueOffset: 0 });
      return undefined;
    }
    case "delete_plant":
      return undefined;
    case "get_weather":
      return settings.weather_enabled && settings.location
        ? {
            location_label: settings.location.label,
            today_max_c: 28.9,
            today_min_c: 21.4,
            recent_rain_mm: 16.3,
            rained_recently: true,
            fetched_at: new Date().toISOString(),
            stale: false,
          }
        : null;
    case "refresh_weather":
      return false;
    case "set_pinned_on_top":
      settings = { ...settings, pinned_on_top: args.pinned as boolean };
      return undefined;
    case "set_active_space":
      settings = { ...settings, active_space_id: (args.spaceId as string | null) ?? null };
      return undefined;
    case "is_autostart_enabled":
      return settings.launch_at_startup;
    case "detect_location":
      return {
        label: "Bengaluru, Karnataka, India",
        latitude: 12.9716,
        longitude: 77.5946,
        timezone: "Asia/Kolkata",
        country_code: "IN",
      };
    case "search_places": {
      const q = ((args.query as string) ?? "").toLowerCase();
      const places = [
        { label: "Bengaluru, Karnataka, India", latitude: 12.97, longitude: 77.59, timezone: "Asia/Kolkata", country_code: "IN" },
        { label: "London, England, United Kingdom", latitude: 51.51, longitude: -0.13, timezone: "Europe/London", country_code: "GB" },
        { label: "Sydney, New South Wales, Australia", latitude: -33.87, longitude: 151.21, timezone: "Australia/Sydney", country_code: "AU" },
      ];
      return places.filter((p) => p.label.toLowerCase().includes(q));
    }
    case "search_catalog": {
      // Small stand-in for the Rust-side bundled catalog.
      const q = ((args.query as string) ?? "").trim().toLowerCase();
      const mock = [
        { id: "curry-leaf", common_name: "Curry Leaf", aliases: ["kadi patta"], scientific_name: "Murraya koenigii", category: "Herb", light: "full_sun", moisture_class: "moderate", fertilize_group: "foliage", typically_hanging: false, uses: "Aromatic leaves for South Indian cooking.", significance: "A kitchen-garden staple in Indian homes.", fun_fact: "It belongs to the citrus family." },
        { id: "golden-pothos", common_name: "Golden Pothos", aliases: ["money plant"], scientific_name: "Epipremnum aureum", category: "Foliage", light: "bright_indirect", moisture_class: "moderate", fertilize_group: "foliage", typically_hanging: true, uses: "Hardy trailing vine.", significance: "Among the most forgiving houseplants.", fun_fact: "Indoors its leaves stay small because it never gets to climb." },
        { id: "tulsi", common_name: "Tulsi", aliases: ["holy basil"], scientific_name: "Ocimum tenuiflorum", category: "Herb", light: "full_sun", moisture_class: "moderate", fertilize_group: "herb_succulent", typically_hanging: false, uses: "Leaves brewed as tea.", significance: "Held sacred in Hinduism.", fun_fact: "Sharper and more clove-like than sweet basil." },
      ];
      if (!q) return mock;
      return mock.filter(
        (m) =>
          m.common_name.toLowerCase().includes(q) ||
          m.aliases.some((a) => a.includes(q)) ||
          m.scientific_name.toLowerCase().includes(q),
      );
    }
    default:
      if (cmd.startsWith("plugin:window|") || cmd.startsWith("plugin:event|")) {
        return undefined;
      }
      console.warn("[preview] unhandled mock IPC command:", cmd, payload);
      return undefined;
  }
}, { shouldMockEvents: true });

const App = React.lazy(() => import("./App"));

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <React.Suspense fallback={null}>
      <App />
    </React.Suspense>
  </React.StrictMode>,
);
