use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// How quickly a plant's soil dries out between checks. Drives the
/// watering check interval per season (see schedule::water_interval_days).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoistureClass {
    /// Keep consistently moist: Passion Fruit, Turtle Vine, Bird of Paradise.
    ConsistentlyMoist,
    /// Let the top 2-3cm dry between checks.
    Moderate,
    /// Allow more drying between waterings.
    Drier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Light {
    FullSun,
    BrightLight,
    BrightIndirect,
}

/// Which feeding schedule a plant follows. See schedule::fertilize_plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FertilizeGroup {
    FloweringFruiting,
    Citrus,
    Foliage,
    HerbSucculent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    HotDry,
    Monsoon,
    Mild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    Water,
    Fertilize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Pending,
    Done,
    Snoozed,
    SkippedSoilWet,
}

/// A named physical area holding plants ("Balcony", "Living room", ...).
/// Stored in spaces.json; every plant belongs to exactly one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: String,
    pub name: String,
}

/// Id of the space that pre-existing plants are migrated into. The app
/// shipped before spaces existed, so any plant record on disk without a
/// `space_id` is by definition one of the original balcony plants.
pub const DEFAULT_SPACE_ID: &str = "balcony";

pub fn default_space_id() -> String {
    DEFAULT_SPACE_ID.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantProfile {
    pub id: String,
    pub common_name: String,
    pub scientific_name: String,
    pub category: String,
    pub light: Light,
    pub moisture_class: MoistureClass,
    pub fertilize_group: FertilizeGroup,
    pub is_hanging: bool,
    pub notes: String,
    /// True when this field was not present in the source inventory and
    /// was filled in with a best-guess default (see plan v2 "Data Gaps").
    /// Surfaced in the UI so the user knows which schedules to double check.
    pub inferred: bool,
    /// Fields below were added after the first release. They all carry
    /// serde defaults so a plants.json written by an older build still
    /// deserializes -- without these, adding a field would make the whole
    /// file fail to parse and silently discard the user's care history.
    #[serde(default = "default_space_id")]
    pub space_id: String,
    /// What the plant is good for -- culinary, medicinal, ornamental.
    #[serde(default)]
    pub uses: String,
    /// Cultural / historical background.
    #[serde(default)]
    pub significance: String,
    /// One-line curiosity, surfaced in the all-clear empty state.
    #[serde(default)]
    pub fun_fact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CareEvent {
    pub id: String,
    pub plant_id: String,
    pub task_type: TaskType,
    pub due_at: NaiveDate,
    pub status: EventStatus,
    pub completed_at: Option<NaiveDate>,
    pub snoozed_until: Option<NaiveDate>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DensityMode {
    Compact,
    Expanded,
}

/// Every field defaults so a settings.json from an older build keeps
/// loading as new settings are introduced (same reasoning as PlantProfile).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// 24h "HH:MM", local (Asia/Kolkata) time the daily digest notification fires.
    pub notification_time: String,
    pub launch_at_startup: bool,
    pub density_mode: DensityMode,
    pub pinned_on_top: bool,
    /// Last calendar date (Asia/Kolkata) a digest notification was sent,
    /// so we don't double-fire if the app restarts within the same day.
    pub last_digest_sent_on: Option<NaiveDate>,
    /// Which space the widget is currently showing. `None` means "all
    /// spaces", which is also what an older settings.json deserializes to.
    pub active_space_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            notification_time: "08:00".to_string(),
            launch_at_startup: true,
            density_mode: DensityMode::Compact,
            pinned_on_top: false,
            last_digest_sent_on: None,
            active_space_id: None,
        }
    }
}
