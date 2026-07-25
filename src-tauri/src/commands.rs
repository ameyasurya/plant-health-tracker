use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::models::{
    CareEvent, EventStatus, FertilizeGroup, Light, MoistureClass, PlantProfile, Settings, Space,
    TaskType, DEFAULT_SPACE_ID,
};
use crate::schedule::{self, ScheduleContext};
use crate::store::{new_pending_event, new_plant_id, new_space_id, Store};
use crate::time::today_local;

pub struct AppState {
    pub store: Mutex<Store>,
}

/// The ambient inputs every scheduling decision needs: the user's local
/// calendar date, their hemisphere, and the cached weather.
///
/// Bundled together because they must be consistent with one another --
/// deriving "today" from one timezone while judging rain from another
/// location would produce subtly wrong due dates.
struct Env {
    today: chrono::NaiveDate,
    latitude: Option<f64>,
    weather: Vec<crate::models::DailyWeather>,
}

impl Env {
    fn load(store: &Store) -> Result<Self, String> {
        let settings = store.load_settings().map_err(|e| e.to_string())?;
        let location = settings.location;
        let weather = if settings.weather_enabled {
            store.load_weather().map_err(|e| e.to_string())?.days
        } else {
            Vec::new()
        };
        Ok(Self {
            today: today_local(location.as_ref()),
            latitude: location.as_ref().map(|l| l.latitude),
            weather,
        })
    }

    fn ctx(&self) -> ScheduleContext<'_> {
        ScheduleContext { latitude: self.latitude, weather: &self.weather }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EventView {
    pub id: String,
    pub plant_id: String,
    pub plant_name: String,
    pub task_type: TaskType,
    pub due_at: chrono::NaiveDate,
    pub days_until: i64,
    /// "overdue" | "today" | "soon"
    pub bucket: String,
    pub cue: String,
    pub instruction: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllPlantsRow {
    pub plant_id: String,
    pub plant_name: String,
    pub scientific_name: String,
    pub next_water: chrono::NaiveDate,
    pub next_water_label: String,
    pub next_fertilize: chrono::NaiveDate,
    pub next_fertilize_label: String,
    pub inferred: bool,
    /// Carried on the row so the all-clear empty state can show a fact
    /// about a plant the user actually owns, with no extra round trip.
    pub fun_fact: String,
}

/// Everything the add-plant form collects. The id, space assignment and
/// the initial care events are derived server-side rather than trusted
/// from the client.
#[derive(Debug, Clone, Deserialize)]
pub struct NewPlant {
    pub common_name: String,
    pub scientific_name: String,
    pub category: String,
    pub light: Light,
    pub moisture_class: MoistureClass,
    pub fertilize_group: FertilizeGroup,
    pub is_hanging: bool,
    pub notes: String,
    pub space_id: String,
    /// Set when the user picked a species from the bundled catalog. The
    /// knowledge copy is looked up server-side from this rather than sent
    /// by the client, so the details panel can't be fed arbitrary text.
    #[serde(default)]
    pub catalog_id: Option<String>,
}

fn cue_label(task_type: TaskType, days_until: i64) -> String {
    let verb = match task_type {
        TaskType::Water => "Water",
        TaskType::Fertilize => "Feed",
    };
    let when = day_label(days_until);
    format!("{verb} · {when}")
}

fn day_label(days_until: i64) -> String {
    if days_until < 0 {
        "overdue".to_string()
    } else if days_until == 0 {
        "today".to_string()
    } else if days_until == 1 {
        "in 1 day".to_string()
    } else {
        format!("in {days_until} days")
    }
}

fn bucket_for(days_until: i64) -> &'static str {
    if days_until < 0 {
        "overdue"
    } else if days_until == 0 {
        "today"
    } else {
        "soon"
    }
}

/// An event counts as "actionable now" once its due date (or, for a
/// snoozed event, its snoozed-until date) has arrived. This is also
/// where the catch-up rule lives: we never look further back than the
/// stored due date, so a multi-day-offline gap shows one overdue item,
/// not a pile of missed-day duplicates.
fn effective_due(event: &CareEvent) -> chrono::NaiveDate {
    match event.status {
        EventStatus::Snoozed => event.snoozed_until.unwrap_or(event.due_at),
        _ => event.due_at,
    }
}

fn is_live(event: &CareEvent) -> bool {
    matches!(event.status, EventStatus::Pending | EventStatus::Snoozed)
}

fn build_event_view(event: &CareEvent, plant: &PlantProfile, today: chrono::NaiveDate) -> EventView {
    let due = effective_due(event);
    let days_until = (due - today).num_days();
    EventView {
        id: event.id.clone(),
        plant_id: plant.id.clone(),
        plant_name: plant.common_name.clone(),
        task_type: event.task_type,
        due_at: due,
        days_until,
        bucket: bucket_for(days_until).to_string(),
        cue: cue_label(event.task_type, days_until),
        instruction: schedule::instruction_for(event.task_type, plant),
    }
}

/// `None` in `Settings.active_space_id` means "show every space", so an
/// unset filter must pass everything rather than matching nothing.
fn in_active_space(plant: &PlantProfile, active: &Option<String>) -> bool {
    match active {
        None => true,
        Some(id) => &plant.space_id == id,
    }
}

fn due_and_upcoming(state: &AppState) -> Result<(Vec<EventView>, Vec<EventView>), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let plants = store.load_plants().map_err(|e| e.to_string())?;
    let events = store.load_events().map_err(|e| e.to_string())?;
    let active = store.load_settings().map_err(|e| e.to_string())?.active_space_id;
    let env = Env::load(&store)?;
    let today = env.today;

    let mut due = Vec::new();
    let mut soon = Vec::new();
    for event in events.iter().filter(|e| is_live(e)) {
        let Some(plant) = plants.iter().find(|p| p.id == event.plant_id) else {
            continue;
        };
        if !in_active_space(plant, &active) {
            continue;
        }
        let view = build_event_view(event, plant, today);
        match view.bucket.as_str() {
            "overdue" | "today" => due.push(view),
            _ if view.days_until <= 5 => soon.push(view),
            _ => {}
        }
    }
    due.sort_by_key(|v| v.days_until);
    soon.sort_by_key(|v| v.days_until);
    Ok((due, soon))
}

#[tauri::command]
pub fn list_due_today(state: tauri::State<AppState>) -> Result<Vec<EventView>, String> {
    Ok(due_and_upcoming(&state)?.0)
}

#[tauri::command]
pub fn list_soon(state: tauri::State<AppState>) -> Result<Vec<EventView>, String> {
    Ok(due_and_upcoming(&state)?.1)
}

#[tauri::command]
pub fn list_all_plants(state: tauri::State<AppState>) -> Result<Vec<AllPlantsRow>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let plants = store.load_plants().map_err(|e| e.to_string())?;
    let events = store.load_events().map_err(|e| e.to_string())?;
    let active = store.load_settings().map_err(|e| e.to_string())?.active_space_id;
    let env = Env::load(&store)?;
    let today = env.today;

    let mut rows = Vec::with_capacity(plants.len());
    for plant in plants.iter().filter(|p| in_active_space(p, &active)) {
        let water = events
            .iter()
            .filter(|e| e.plant_id == plant.id && e.task_type == TaskType::Water && is_live(e))
            .map(effective_due)
            .min()
            .unwrap_or(today);
        let fert = events
            .iter()
            .filter(|e| e.plant_id == plant.id && e.task_type == TaskType::Fertilize && is_live(e))
            .map(effective_due)
            .min()
            .unwrap_or(today);
        rows.push(AllPlantsRow {
            plant_id: plant.id.clone(),
            plant_name: plant.common_name.clone(),
            scientific_name: plant.scientific_name.clone(),
            next_water: water,
            next_water_label: day_label((water - today).num_days()),
            next_fertilize: fert,
            next_fertilize_label: day_label((fert - today).num_days()),
            inferred: plant.inferred,
            fun_fact: plant.fun_fact.clone(),
        });
    }
    rows.sort_by(|a, b| a.plant_name.cmp(&b.plant_name));
    Ok(rows)
}

/// Marks the given event as done and opens a fresh Pending event for the
/// next occurrence, computed from today (not from the old due date) --
/// this is the catch-up rule: a stale/overdue event never chains extra
/// cycles just because it sat unresolved for a while.
#[tauri::command]
pub fn mark_done(state: tauri::State<AppState>, event_id: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let plants = store.load_plants().map_err(|e| e.to_string())?;
    let mut events = store.load_events().map_err(|e| e.to_string())?;
    let env = Env::load(&store)?;
    let today = env.today;

    let idx = events
        .iter()
        .position(|e| e.id == event_id)
        .ok_or_else(|| "event not found".to_string())?;
    let plant = plants
        .iter()
        .find(|p| p.id == events[idx].plant_id)
        .ok_or_else(|| "plant not found".to_string())?
        .clone();

    let task_type = events[idx].task_type;
    let next = schedule::next_due_ctx(today, &plant, task_type, env.ctx());

    events[idx].status = EventStatus::Done;
    events[idx].completed_at = Some(today);
    events.push(new_pending_event(plant.id.clone(), task_type, next));

    store.save_events(&events).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn snooze(state: tauri::State<AppState>, event_id: String, days: i64) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut events = store.load_events().map_err(|e| e.to_string())?;
    let env = Env::load(&store)?;
    let today = env.today;
    let idx = events
        .iter()
        .position(|e| e.id == event_id)
        .ok_or_else(|| "event not found".to_string())?;
    events[idx].status = EventStatus::Snoozed;
    events[idx].snoozed_until = Some(today + chrono::Duration::days(days.max(1)));
    store.save_events(&events).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn skip_soil_wet(state: tauri::State<AppState>, event_id: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let plants = store.load_plants().map_err(|e| e.to_string())?;
    let mut events = store.load_events().map_err(|e| e.to_string())?;
    let env = Env::load(&store)?;
    let today = env.today;
    let idx = events
        .iter()
        .position(|e| e.id == event_id)
        .ok_or_else(|| "event not found".to_string())?;
    let plant = plants
        .iter()
        .find(|p| p.id == events[idx].plant_id)
        .ok_or_else(|| "plant not found".to_string())?
        .clone();

    let next = schedule::skip_recheck_due_ctx(today, &plant, env.ctx());
    let task_type = events[idx].task_type;
    let plant_id = events[idx].plant_id.clone();

    events[idx].status = EventStatus::SkippedSoilWet;
    events[idx].completed_at = Some(today);
    events.push(new_pending_event(plant_id, task_type, next));

    store.save_events(&events).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_plant(state: tauri::State<AppState>, plant: PlantProfile) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut plants = store.load_plants().map_err(|e| e.to_string())?;
    if let Some(existing) = plants.iter_mut().find(|p| p.id == plant.id) {
        *existing = plant;
    } else {
        return Err("plant not found".to_string());
    }
    store.save_plants(&plants).map_err(|e| e.to_string())?;
    Ok(())
}

/// Full profile for the details / edit panels. `list_all_plants` returns a
/// reduced projection, so this is the only way to get the care-class fields
/// and knowledge blurbs for a single plant.
#[tauri::command]
pub fn get_plant(state: tauri::State<AppState>, plant_id: String) -> Result<PlantProfile, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let plants = store.load_plants().map_err(|e| e.to_string())?;
    plants
        .into_iter()
        .find(|p| p.id == plant_id)
        .ok_or_else(|| "plant not found".to_string())
}

/// Adds a plant and opens its first Water + Fertilize events, mirroring
/// what `ensure_seeded` does for the built-in plants so a new plant shows
/// up in the Today/Soon lists straight away.
#[tauri::command]
pub fn add_plant(state: tauri::State<AppState>, plant: NewPlant) -> Result<PlantProfile, String> {
    let name = plant.common_name.trim();
    if name.is_empty() {
        return Err("plant needs a name".to_string());
    }

    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut plants = store.load_plants().map_err(|e| e.to_string())?;
    let mut events = store.load_events().map_err(|e| e.to_string())?;
    let spaces = store.load_spaces().map_err(|e| e.to_string())?;

    let space_id = if spaces.iter().any(|s| s.id == plant.space_id) {
        plant.space_id.clone()
    } else {
        DEFAULT_SPACE_ID.to_string()
    };

    // Knowledge copy comes from the bundled catalog when the user picked a
    // species, so a plant added as "Curry Leaf" still gets its details and
    // can appear in the all-clear fun fact.
    let known = plant.catalog_id.as_deref().and_then(crate::catalog::get);

    let profile = PlantProfile {
        id: new_plant_id(name),
        common_name: name.to_string(),
        scientific_name: plant.scientific_name.trim().to_string(),
        category: plant.category.trim().to_string(),
        light: plant.light,
        moisture_class: plant.moisture_class,
        fertilize_group: plant.fertilize_group,
        is_hanging: plant.is_hanging,
        notes: plant.notes.trim().to_string(),
        inferred: false,
        space_id,
        uses: known.map(|k| k.uses.clone()).unwrap_or_default(),
        significance: known.map(|k| k.significance.clone()).unwrap_or_default(),
        fun_fact: known.map(|k| k.fun_fact.clone()).unwrap_or_default(),
    };

    let env = Env::load(&store)?;
    let today = env.today;
    events.push(new_pending_event(profile.id.clone(), TaskType::Water, today));
    let fert_due = schedule::next_fertilize_due_ctx(today, &profile, env.ctx());
    events.push(new_pending_event(profile.id.clone(), TaskType::Fertilize, fert_due));

    plants.push(profile.clone());
    store.save_plants(&plants).map_err(|e| e.to_string())?;
    store.save_events(&events).map_err(|e| e.to_string())?;
    Ok(profile)
}

/// Removes a plant and every care event that references it, so the log
/// doesn't accumulate orphaned rows pointing at a plant that no longer exists.
#[tauri::command]
pub fn delete_plant(state: tauri::State<AppState>, plant_id: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut plants = store.load_plants().map_err(|e| e.to_string())?;
    let before = plants.len();
    plants.retain(|p| p.id != plant_id);
    if plants.len() == before {
        return Err("plant not found".to_string());
    }
    let mut events = store.load_events().map_err(|e| e.to_string())?;
    events.retain(|e| e.plant_id != plant_id);
    store.save_plants(&plants).map_err(|e| e.to_string())?;
    store.save_events(&events).map_err(|e| e.to_string())?;
    Ok(())
}

/// Species lookup for the add-plant form. Read-only and offline -- the
/// catalog is compiled into the binary.
#[tauri::command]
pub fn search_catalog(query: String, limit: Option<usize>) -> Vec<&'static crate::catalog::CatalogEntry> {
    crate::catalog::search(&query, limit.unwrap_or(8))
}

#[tauri::command]
pub fn list_spaces(state: tauri::State<AppState>) -> Result<Vec<Space>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.load_spaces().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_space(state: tauri::State<AppState>, name: String) -> Result<Space, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("space needs a name".to_string());
    }
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut spaces = store.load_spaces().map_err(|e| e.to_string())?;
    if spaces.iter().any(|s| s.name.eq_ignore_ascii_case(&name)) {
        return Err(format!("a space called \"{name}\" already exists"));
    }
    let space = Space { id: new_space_id(&name), name };
    spaces.push(space.clone());
    store.save_spaces(&spaces).map_err(|e| e.to_string())?;
    Ok(space)
}

#[tauri::command]
pub fn rename_space(state: tauri::State<AppState>, space_id: String, name: String) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("space needs a name".to_string());
    }
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut spaces = store.load_spaces().map_err(|e| e.to_string())?;
    if spaces
        .iter()
        .any(|s| s.id != space_id && s.name.eq_ignore_ascii_case(&name))
    {
        return Err(format!("a space called \"{name}\" already exists"));
    }
    let Some(space) = spaces.iter_mut().find(|s| s.id == space_id) else {
        return Err("space not found".to_string());
    };
    space.name = name;
    store.save_spaces(&spaces).map_err(|e| e.to_string())?;
    Ok(())
}

/// Deleting a space never deletes plants -- they are moved into another
/// space instead, so a mis-click can't destroy care history. The last
/// remaining space can't be deleted since plants must live somewhere.
#[tauri::command]
pub fn delete_space(state: tauri::State<AppState>, space_id: String) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut spaces = store.load_spaces().map_err(|e| e.to_string())?;
    if spaces.len() <= 1 {
        return Err("can't delete the only space".to_string());
    }
    if !spaces.iter().any(|s| s.id == space_id) {
        return Err("space not found".to_string());
    }
    spaces.retain(|s| s.id != space_id);
    let fallback = spaces[0].id.clone();

    let mut plants = store.load_plants().map_err(|e| e.to_string())?;
    for plant in plants.iter_mut().filter(|p| p.space_id == space_id) {
        plant.space_id = fallback.clone();
    }

    let mut settings = store.load_settings().map_err(|e| e.to_string())?;
    if settings.active_space_id.as_deref() == Some(space_id.as_str()) {
        settings.active_space_id = None;
        store.save_settings(&settings).map_err(|e| e.to_string())?;
    }

    store.save_plants(&plants).map_err(|e| e.to_string())?;
    store.save_spaces(&spaces).map_err(|e| e.to_string())?;
    Ok(())
}

// ---- Location & weather ---------------------------------------------

/// City search. Network call, so it's async and never blocks the UI.
#[tauri::command]
pub async fn search_places(query: String) -> Result<Vec<crate::models::Location>, String> {
    crate::weather::search_places(&query).await
}

/// Approximate location from the caller's IP.
///
/// Only ever invoked from an explicit "Detect" button -- see the note in
/// weather::detect_location about not phoning home silently.
#[tauri::command]
pub async fn detect_location() -> Result<crate::models::Location, String> {
    crate::weather::detect_location().await
}

/// Fetches the forecast for the saved location and caches it.
///
/// Returns `Ok(false)` rather than an error when there's nothing to do
/// (weather disabled, or no location yet) so the UI can call this freely
/// on startup without special-casing.
#[tauri::command]
pub async fn refresh_weather(state: tauri::State<'_, AppState>, force: bool) -> Result<bool, String> {
    // Read what we need, then drop the lock before awaiting -- holding a
    // std::sync::Mutex across an await point would block every other
    // command for the duration of a network round trip.
    let (location, enabled, cached) = {
        let store = state.store.lock().map_err(|e| e.to_string())?;
        let settings = store.load_settings().map_err(|e| e.to_string())?;
        let cache = store.load_weather().map_err(|e| e.to_string())?;
        (settings.location, settings.weather_enabled, cache)
    };

    let Some(location) = location else { return Ok(false) };
    if !enabled {
        return Ok(false);
    }
    if !force && !cache_is_stale(&cached, &location) {
        return Ok(false);
    }

    let days = crate::weather::fetch_forecast(location.latitude, location.longitude).await?;

    let store = state.store.lock().map_err(|e| e.to_string())?;
    store
        .save_weather(&crate::models::WeatherCache {
            fetched_at: Some(chrono::Utc::now().to_rfc3339()),
            latitude: Some(location.latitude),
            longitude: Some(location.longitude),
            days,
        })
        .map_err(|e| e.to_string())?;
    Ok(true)
}

/// Refetch when the data is older than a few hours, or when the user has
/// moved far enough that the old forecast is for somewhere else.
fn cache_is_stale(cache: &crate::models::WeatherCache, location: &crate::models::Location) -> bool {
    const MAX_AGE_HOURS: i64 = 6;
    const MOVED_DEGREES: f64 = 0.5;

    let moved = match (cache.latitude, cache.longitude) {
        (Some(la), Some(lo)) => {
            (la - location.latitude).abs() > MOVED_DEGREES
                || (lo - location.longitude).abs() > MOVED_DEGREES
        }
        _ => true,
    };
    if moved || cache.days.is_empty() {
        return true;
    }
    match cache.fetched_at.as_deref().and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()) {
        Some(t) => chrono::Utc::now().signed_duration_since(t).num_hours() >= MAX_AGE_HOURS,
        None => true,
    }
}

/// Weather strip for the UI. `None` when weather is off or unconfigured.
#[tauri::command]
pub fn get_weather(state: tauri::State<AppState>) -> Result<Option<crate::weather::WeatherSummary>, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let settings = store.load_settings().map_err(|e| e.to_string())?;
    let Some(location) = settings.location.as_ref() else { return Ok(None) };
    if !settings.weather_enabled {
        return Ok(None);
    }
    let cache = store.load_weather().map_err(|e| e.to_string())?;
    if cache.days.is_empty() {
        return Ok(None);
    }

    let today = today_local(Some(location));
    let today_row = cache.days.iter().find(|d| d.date == today);
    let adj = schedule::weather_adjustment(today, &cache.days, schedule::WEATHER_LOOKBACK_DAYS);
    let recent_rain: f64 = cache
        .days
        .iter()
        .filter(|d| d.date > today - chrono::Duration::days(schedule::WEATHER_LOOKBACK_DAYS) && d.date <= today)
        .map(|d| d.precipitation_mm)
        .sum();

    let stale = cache
        .fetched_at
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|t| chrono::Utc::now().signed_duration_since(t).num_hours() >= 12)
        .unwrap_or(true);

    Ok(Some(crate::weather::WeatherSummary {
        location_label: location.label.clone(),
        today_max_c: today_row.map(|d| d.temp_max_c).filter(|v| !v.is_nan()),
        today_min_c: today_row.map(|d| d.temp_min_c).filter(|v| !v.is_nan()),
        recent_rain_mm: (recent_rain * 10.0).round() / 10.0,
        rained_recently: adj.reason == Some("recent rain"),
        fetched_at: cache.fetched_at.clone(),
        stale,
    }))
}

#[tauri::command]
pub fn get_settings(state: tauri::State<AppState>) -> Result<Settings, String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.load_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(state: tauri::State<AppState>, settings: Settings) -> Result<(), String> {
    let store = state.store.lock().map_err(|e| e.to_string())?;
    store.save_settings(&settings).map_err(|e| e.to_string())
}
