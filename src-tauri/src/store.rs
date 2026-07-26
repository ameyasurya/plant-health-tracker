//! Local JSON persistence: plants.json, care-log.json, spaces.json,
//! settings.json, weather.json and todos.json.
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
    CareEvent, EventStatus, PlantProfile, Settings, Space, TaskType, Todo, WeatherCache,
    DEFAULT_SPACE_ID,
};

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

    /// Directory holding the JSON files. Exposed so tests can plant a
    /// stray temp file alongside them.
    pub fn dir(&self) -> &PathBuf {
        &self.dir
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

    /// Checklist items. A fresh install has no todos.json and simply
    /// starts with an empty list; there is nothing sensible to seed.
    pub fn load_todos(&self) -> StoreResult<Vec<Todo>> {
        Ok(self.read("todos.json")?.unwrap_or_default())
    }

    pub fn save_todos(&self, todos: &[Todo]) -> StoreResult<()> {
        self.write_atomic("todos.json", &todos)
    }

    /// Cached forecast. Persisted rather than held in memory so the
    /// schedule still reflects recent rain after a restart with no
    /// network -- offline behaviour is the point of caching it at all.
    pub fn load_weather(&self) -> StoreResult<WeatherCache> {
        Ok(self.read("weather.json")?.unwrap_or_default())
    }

    pub fn save_weather(&self, cache: &WeatherCache) -> StoreResult<()> {
        self.write_atomic("weather.json", cache)
    }

    /// First-run initialisation.
    ///
    /// Deliberately creates NO plants. Earlier builds seeded the author's
    /// own 19 balcony plants, which is wrong for anyone else installing
    /// this -- a new user should start empty and add their own via the
    /// bundled species catalog. Only the containers (an empty plant list,
    /// one default space, default settings) are written.
    ///
    /// No-op once plants.json exists, so existing installs are untouched.
    pub fn ensure_initialised(&self) -> StoreResult<()> {
        if self.path("plants.json").exists() {
            return Ok(());
        }
        self.save_plants(&[])?;
        self.save_events(&[])?;
        self.save_spaces(&[default_space()])?;
        self.save_settings(&Settings::default())?;
        Ok(())
    }

    /// Fills in uses/significance/fun_fact on plants that predate those
    /// fields, matching against the species catalog by id.
    ///
    /// Installs created by an early build have plants with empty knowledge
    /// copy, and since first-run init no-ops once plants.json exists they
    /// would stay blank forever -- leaving the details panel and the
    /// all-clear fun fact looking broken. Only empty fields are written,
    /// so a user's own edits are never overwritten, and this is a no-op on
    /// every run after the first.
    pub fn backfill_catalog_knowledge(&self) -> StoreResult<()> {
        let mut plants = self.load_plants()?;
        let mut changed = false;

        for plant in plants.iter_mut() {
            let Some(entry) = crate::catalog::get(&plant.id) else {
                continue;
            };
            if plant.uses.is_empty() && !entry.uses.is_empty() {
                plant.uses = entry.uses.clone();
                changed = true;
            }
            if plant.significance.is_empty() && !entry.significance.is_empty() {
                plant.significance = entry.significance.clone();
                changed = true;
            }
            if plant.fun_fact.is_empty() && !entry.fun_fact.is_empty() {
                plant.fun_fact = entry.fun_fact.clone();
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

pub fn new_todo(text: String, created_on: chrono::NaiveDate) -> Todo {
    Todo {
        id: Uuid::new_v4().to_string(),
        text,
        done: false,
        created_on,
        completed_on: None,
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
