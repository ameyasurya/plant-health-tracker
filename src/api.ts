import { invoke } from "@tauri-apps/api/core";
import type { AllPlantsRow, EventView, NewPlant, PlantProfile, Settings, Space } from "./types";

export const api = {
  listDueToday: () => invoke<EventView[]>("list_due_today"),
  listSoon: () => invoke<EventView[]>("list_soon"),
  listAllPlants: () => invoke<AllPlantsRow[]>("list_all_plants"),
  markDone: (eventId: string) => invoke<void>("mark_done", { eventId }),
  snooze: (eventId: string, days: number) => invoke<void>("snooze", { eventId, days }),
  skipSoilWet: (eventId: string) => invoke<void>("skip_soil_wet", { eventId }),
  getPlant: (plantId: string) => invoke<PlantProfile>("get_plant", { plantId }),
  addPlant: (plant: NewPlant) => invoke<PlantProfile>("add_plant", { plant }),
  updatePlant: (plant: PlantProfile) => invoke<void>("update_plant", { plant }),
  deletePlant: (plantId: string) => invoke<void>("delete_plant", { plantId }),
  listSpaces: () => invoke<Space[]>("list_spaces"),
  addSpace: (name: string) => invoke<Space>("add_space", { name }),
  renameSpace: (spaceId: string, name: string) => invoke<void>("rename_space", { spaceId, name }),
  deleteSpace: (spaceId: string) => invoke<void>("delete_space", { spaceId }),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) => invoke<void>("update_settings", { settings }),
};
