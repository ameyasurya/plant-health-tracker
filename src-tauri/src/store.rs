//! Local JSON persistence for plants.json, care-log.json and settings.json.
//!
//! Every write goes through `write_atomic`: serialize to a sibling `.tmp`
//! file, flush, then rename over the real file. Rename is atomic on both
//! NTFS and the Tauri app-data volume, so a crash or power loss mid-write
//! can never leave a half-written, corrupted JSON file behind -- the
//! reader either sees the old complete file or the new complete file,
//! never a partial one.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::models::{
    CareEvent, EventStatus, PlantProfile, Settings, Space, TaskType, DEFAULT_SPACE_ID,
};
use crate::schedule;
use crate::seed;
use crate::time::today_ist;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type StoreResult<T> = Result<T, StoreError>;

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn new(app_data_dir: PathBuf) -> StoreResult<Self> {
        fs::create_dir_all(&app_data_dir)?;
        Ok(Self { dir: app_data_dir })
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    fn read<T: DeserializeOwned>(&self, name: &str) -> StoreResult<Option<T>> {
        let path = self.path(name);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path)?;
        let value = serde_json::from_slice(&bytes)?;
        Ok(Some(value))
    }

    fn write_atomic<T: Serialize>(&self, name: &str, value: &T) -> StoreResult<()> {
        let path = self.path(name);
        let tmp_path = self.path(&format!("{name}.{}.tmp", Uuid::new_v4()));
        let json = serde_json::to_vec_pretty(value)?;
        {
            let mut file = fs::File::create(&tmp_path)?;
            file.write_all(&json)?;
            file.sync_all()?;
        }
        fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    pub fn load_plants(&self) -> StoreResult<Vec<PlantProfile>> {
        Ok(self.read("plants.json")?.unwrap_or_default())
    }

    pub fn save_plants(&self, plants: &[PlantProfile]) -> StoreResult<()> {
        self.write_atomic("plants.json", &plants)
    }

    pub fn load_events(&self) -> StoreResult<Vec<CareEvent>> {
        Ok(self.read("care-log.json")?.unwrap_or_default())
    }

    pub fn save_events(&self, events: &[CareEvent]) -> StoreResult<()> {
        self.write_atomic("care-log.json", &events)
    }

    pub fn load_settings(&self) -> StoreResult<Settings> {
        Ok(self.read("settings.json")?.unwrap_or_default())
    }

    pub fn save_settings(&self, settings: &Settings) -> StoreResult<()> {
        self.write_atomic("settings.json", settings)
    }

    /// Spaces were added after the first release. An install that predates
    /// them has no spaces.json at all, so synthesise the one space its
    /// plants are already implicitly in rather than returning an empty list
    /// (which would leave the UI with nowhere to put existing plants).
    pub fn load_spaces(&self) -> StoreResult<Vec<Space>> {
        let spaces: Vec<Space> = self.read("spaces.json")?.unwrap_or_default();
        if spaces.is_empty() {
            return Ok(vec![default_space()]);
        }
        Ok(spaces)
    }

    pub fn save_spaces(&self, spaces: &[Space]) -> StoreResult<()> {
        self.write_atomic("spaces.json", &spaces)
    }

    /// First-run seed: writes the 19 plants from the inventory and one
    /// initial pending Water + Fertilize event per plant. No-op if
    /// plants.json already exists -- there is no ongoing "import" flow,
    /// this runs exactly once per install.
    pub fn ensure_seeded(&self) -> StoreResult<()> {
        if self.path("plants.json").exists() {
            return Ok(());
        }
        let plants = seed::seed_plants();
        let today = today_ist();
        let mut events = Vec::with_capacity(plants.len() * 2);
        for plant in &plants {
            events.push(new_pending_event(plant.id.clone(), TaskType::Water, today));
            let fert_due = schedule::next_fertilize_due(today, plant);
            events.push(new_pending_event(plant.id.clone(), TaskType::Fertilize, fert_due));
        }
        self.save_plants(&plants)?;
        self.save_events(&events)?;
        self.save_spaces(&[default_space()])?;
        self.save_settings(&Settings::default())?;
        Ok(())
    }

    /// Fills in the uses/significance/fun_fact blurbs on plants that were
    /// seeded before those fields existed.
    ///
    /// `ensure_seeded` deliberately no-ops once plants.json exists, so an
    /// install from an earlier build would otherwise keep its plants
    /// forever blank and the details panel and all-clear fun fact would
    /// look broken on real data. Matching is by seed id, and only empty
    /// fields are written, so anything the user has typed themselves is
    /// left alone and this stays a no-op on every run after the first.
    pub fn backfill_seed_knowledge(&self) -> StoreResult<()> {
        let mut plants = self.load_plants()?;
        let seeds = seed::seed_plants();
        let mut changed = false;

        for plant in plants.iter_mut() {
            let Some(seed_plant) = seeds.iter().find(|s| s.id == plant.id) else {
                continue;
            };
            if plant.uses.is_empty() && !seed_plant.uses.is_empty() {
                plant.uses = seed_plant.uses.clone();
                changed = true;
            }
            if plant.significance.is_empty() && !seed_plant.significance.is_empty() {
                plant.significance = seed_plant.significance.clone();
                changed = true;
            }
            if plant.fun_fact.is_empty() && !seed_plant.fun_fact.is_empty() {
                plant.fun_fact = seed_plant.fun_fact.clone();
                changed = true;
            }
        }

        if changed {
            self.save_plants(&plants)?;
        }
        Ok(())
    }
}

pub fn default_space() -> Space {
    Space {
        id: DEFAULT_SPACE_ID.to_string(),
        name: "Balcony".to_string(),
    }
}

pub fn new_plant_id(common_name: &str) -> String {
    let slug: String = common_name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    // Always suffix: two plants can legitimately share a common name
    // ("Jasmine" in two different spaces), and ids must stay unique
    // because care events reference plants by id.
    let short = Uuid::new_v4().to_string()[..8].to_string();
    if slug.is_empty() {
        format!("plant-{short}")
    } else {
        format!("{slug}-{short}")
    }
}

pub fn new_space_id(name: &str) -> String {
    let slug: String = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let slug = slug.trim_matches('-').to_string();
    let short = Uuid::new_v4().to_string()[..8].to_string();
    if slug.is_empty() {
        format!("space-{short}")
    } else {
        format!("{slug}-{short}")
    }
}

pub fn new_pending_event(plant_id: String, task_type: TaskType, due_at: chrono::NaiveDate) -> CareEvent {
    CareEvent {
        id: Uuid::new_v4().to_string(),
        plant_id,
        task_type,
        due_at,
        status: EventStatus::Pending,
        completed_at: None,
        snoozed_until: None,
        note: None,
    }
}
