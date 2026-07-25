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
}

export type Light = "full_sun" | "bright_light" | "bright_indirect";
export type MoistureClass = "consistently_moist" | "moderate" | "drier";
export type FertilizeGroup = "flowering_fruiting" | "citrus" | "foliage" | "herb_succulent";

export interface Space {
  id: string;
  name: string;
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
}

export type Tab = "today" | "soon" | "all";

export type MascotState = "happy" | "content" | "worried" | "wilted";
