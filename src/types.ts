export type TaskType = "water" | "fertilize";
export type Bucket = "overdue" | "today" | "soon";

export interface EventView {
  id: string;
  plant_id: string;
  plant_name: string;
  task_type: TaskType;
  due_at: string;
  days_until: number;
  bucket: Bucket;
  cue: string;
  instruction: string;
}

export interface AllPlantsRow {
  plant_id: string;
  plant_name: string;
  scientific_name: string;
  next_water: string;
  next_water_label: string;
  next_fertilize: string;
  next_fertilize_label: string;
  inferred: boolean;
  fun_fact: string;
  space_id: string;
  space_name: string;
  water_status: Bucket;
  fertilize_status: Bucket;
}

/** A checklist item, with the carried-over judgement already made in Rust
 *  against the user's configured timezone rather than the machine's. */
export interface TodoView {
  id: string;
  text: string;
  done: boolean;
  created_on: string;
  carried_over: boolean;
  /** "" for today's items, else "from yesterday" / "from 3 days ago". */
  age_label: string;
}

export type Light = "full_sun" | "bright_light" | "bright_indirect";
export type MoistureClass = "consistently_moist" | "moderate" | "drier";
export type FertilizeGroup = "flowering_fruiting" | "citrus" | "foliage" | "herb_succulent";

export interface Space {
  id: string;
  name: string;
}

export interface Location {
  label: string;
  latitude: number;
  longitude: number;
  /** IANA name, e.g. "Asia/Kolkata". */
  timezone: string;
  country_code: string;
}

export interface WeatherSummary {
  location_label: string;
  today_max_c: number | null;
  today_min_c: number | null;
  recent_rain_mm: number;
  rained_recently: boolean;
  fetched_at: string | null;
  stale: boolean;
}

export interface PlantProfile {
  id: string;
  common_name: string;
  scientific_name: string;
  category: string;
  light: Light;
  moisture_class: MoistureClass;
  fertilize_group: FertilizeGroup;
  is_hanging: boolean;
  notes: string;
  inferred: boolean;
  space_id: string;
  uses: string;
  significance: string;
  fun_fact: string;
}

export interface NewPlant {
  common_name: string;
  scientific_name: string;
  category: string;
  light: Light;
  moisture_class: MoistureClass;
  fertilize_group: FertilizeGroup;
  is_hanging: boolean;
  notes: string;
  space_id: string;
  /** Set when the species came from the bundled catalog; the backend uses
   *  it to attach the uses/significance/fun-fact copy. */
  catalog_id?: string | null;
  /** Days since the plant was last watered / fed, or null for "not sure".
   *  A day count rather than a date, because only the backend knows which
   *  timezone "today" should be resolved in. */
  last_watered_days_ago?: number | null;
  last_fertilized_days_ago?: number | null;
}

/** A species in the bundled catalog (see src-tauri/data/catalog.json). */
export interface CatalogEntry {
  id: string;
  common_name: string;
  aliases: string[];
  scientific_name: string;
  category: string;
  light: Light;
  moisture_class: MoistureClass;
  fertilize_group: FertilizeGroup;
  typically_hanging: boolean;
  uses: string;
  significance: string;
  fun_fact: string;
}

export const LIGHT_OPTIONS: { value: Light; label: string }[] = [
  { value: "full_sun", label: "Full sun" },
  { value: "bright_light", label: "Bright light" },
  { value: "bright_indirect", label: "Bright indirect" },
];

export const MOISTURE_OPTIONS: { value: MoistureClass; label: string }[] = [
  { value: "consistently_moist", label: "Keep moist" },
  { value: "moderate", label: "Let top dry" },
  { value: "drier", label: "Let dry out" },
];

export const FERTILIZE_OPTIONS: { value: FertilizeGroup; label: string }[] = [
  { value: "flowering_fruiting", label: "Flowering / fruiting" },
  { value: "citrus", label: "Citrus" },
  { value: "foliage", label: "Foliage" },
  { value: "herb_succulent", label: "Herb / succulent" },
];

export type DensityMode = "compact" | "expanded";

export interface Settings {
  notification_time: string;
  launch_at_startup: boolean;
  density_mode: DensityMode;
  pinned_on_top: boolean;
  last_digest_sent_on: string | null;
  /** null means "show every space". */
  active_space_id: string | null;
  /** null until location setup is completed. */
  location: Location | null;
  weather_enabled: boolean;
}

export type Tab = "today" | "soon" | "overview" | "todo";

export type MascotState = "happy" | "content" | "worried" | "wilted";
