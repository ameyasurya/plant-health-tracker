//! Test plan coverage:
//!   - schedule generation for every catalog species x 3 seasons
//!   - month-boundary season transitions
//!   - done / snooze / skip-soil-wet recompute next due date correctly
//!   - multi-day-offline catch-up does not backlog/duplicate
//!   - atomic write persists correctly and survives a stray leftover .tmp
//!
//! These exercise the same schedule + store logic the Tauri commands call,
//! without needing a running Tauri app/webview (which needs the heavier
//! `tauri::test` mock runtime) -- the business logic under test is
//! identical either way.

use chrono::{Datelike, Duration, NaiveDate};

use plant_health_tracker_lib::models::{EventStatus, TaskType};
use plant_health_tracker_lib::schedule::{
    next_due, next_fertilize_due, next_water_due, season_for_month, skip_recheck_due,
    water_interval_days,
};
use plant_health_tracker_lib::catalog;
use plant_health_tracker_lib::store::{new_pending_event, Store};

fn temp_store() -> Store {
    let dir = std::env::temp_dir().join(format!("plant-health-tracker-test-{}", uuid::Uuid::new_v4()));
    Store::new(dir).expect("create temp store")
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// A store populated with every catalog species and an initial water +
/// fertilize event each.
///
/// First-run init deliberately creates nothing now (a new user starts
/// empty and adds their own plants), so tests that need existing data
/// build it explicitly instead of leaning on the app's startup path.
fn store_with_plants(today: NaiveDate) -> Store {
    let store = temp_store();
    store.ensure_initialised().unwrap();

    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    let mut events = Vec::with_capacity(plants.len() * 2);
    for plant in &plants {
        events.push(new_pending_event(plant.id.clone(), TaskType::Water, today));
        let fert = next_fertilize_due(today, plant);
        events.push(new_pending_event(plant.id.clone(), TaskType::Fertilize, fert));
    }
    store.save_plants(&plants).unwrap();
    store.save_events(&events).unwrap();
    store
}

// ---- Season month-boundary transitions ----

#[test]
fn season_boundaries_are_correct() {
    use plant_health_tracker_lib::models::Season::*;
    assert_eq!(season_for_month(2), Mild); // Feb 28/29 -> Mar 1
    assert_eq!(season_for_month(3), HotDry);
    assert_eq!(season_for_month(5), HotDry); // May 31 -> Jun 1
    assert_eq!(season_for_month(6), Monsoon);
    assert_eq!(season_for_month(10), Monsoon); // Oct 31 -> Nov 1
    assert_eq!(season_for_month(11), Mild);
    assert_eq!(season_for_month(12), Mild);
    assert_eq!(season_for_month(1), Mild);
}

// ---- Schedule generation for all 19 plants x 3 seasons ----

#[test]
fn every_plant_has_a_positive_water_interval_in_every_season() {
    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    assert!(plants.len() >= 40, "catalog should cover a broad set of species");

    let sample_dates_per_season = [
        date(2026, 4, 10),  // hot/dry
        date(2026, 8, 10),  // monsoon
        date(2026, 12, 10), // mild
    ];

    for plant in &plants {
        for &today in &sample_dates_per_season {
            let season = season_for_month(today.month());
            let interval = water_interval_days(plant.moisture_class, season, plant.is_hanging);
            assert!(
                interval >= 1,
                "{} should have a >=1 day water interval in {:?}",
                plant.common_name,
                season
            );
            let due = next_water_due(today, plant);
            assert_eq!(due, today + Duration::days(interval));
        }
    }
}

#[test]
fn hanging_plants_get_shorter_or_equal_interval_than_potted_equivalent() {
    // Hanging plants dry faster, so for the same moisture class the
    // hanging interval should never exceed the potted one.
    use plant_health_tracker_lib::models::{MoistureClass, Season};
    for &season in &[Season::HotDry, Season::Monsoon, Season::Mild] {
        for &moisture in &[
            MoistureClass::ConsistentlyMoist,
            MoistureClass::Moderate,
            MoistureClass::Drier,
        ] {
            let hanging = water_interval_days(moisture, season, true);
            let potted = water_interval_days(moisture, season, false);
            assert!(hanging <= potted);
        }
    }
}

#[test]
fn fertilizing_pauses_outside_active_window_and_resumes_correctly() {
    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    let ficus = plants.iter().find(|p| p.id == "ficus-benjamina").unwrap(); // Foliage group, active Mar-Oct
    let due = next_fertilize_due(date(2026, 11, 15), ficus);
    assert_eq!(due, date(2027, 3, 1), "foliage feed should resume March 1 after the Nov-Feb pause");

    let orange = plants.iter().find(|p| p.id == "orange").unwrap(); // Citrus group, active Feb-Oct
    let due = next_fertilize_due(date(2026, 11, 15), orange);
    assert_eq!(due, date(2027, 2, 1), "citrus feed should resume Feb 1, one month earlier than foliage");
}

// ---- Done / snooze / skip-soil-wet flows recompute correctly ----

#[test]
fn mark_done_schedules_next_occurrence_from_today_not_from_old_due_date() {
    let store = store_with_plants(date(2026, 4, 20));
    let plants = store.load_plants().unwrap();
    let mut events = store.load_events().unwrap();

    let plant = plants.iter().find(|p| p.id == "jasmine").unwrap();
    let idx = events
        .iter()
        .position(|e| e.plant_id == plant.id && e.task_type == TaskType::Water)
        .unwrap();

    // Simulate the event having gone stale (overdue by 10 days).
    let today = date(2026, 4, 20);
    events[idx].due_at = today - Duration::days(10);

    let next = next_due(today, plant, TaskType::Water);
    events[idx].status = EventStatus::Done;
    events[idx].completed_at = Some(today);
    events.push(new_pending_event(plant.id.clone(), TaskType::Water, next));
    store.save_events(&events).unwrap();

    let reloaded = store.load_events().unwrap();
    let fresh = reloaded
        .iter()
        .find(|e| e.plant_id == plant.id && e.task_type == TaskType::Water && e.status == EventStatus::Pending)
        .expect("a fresh pending event should exist");
    assert!(fresh.due_at > today, "next occurrence must be in the future relative to today");
    assert_eq!(fresh.due_at, next_water_due(today, plant));
}

#[test]
fn snooze_pushes_due_date_out_without_creating_duplicate_events() {
    let store = store_with_plants(date(2026, 4, 20));
    let mut events = store.load_events().unwrap();
    let before_count = events.len();

    let idx = events
        .iter()
        .position(|e| e.task_type == TaskType::Water)
        .unwrap();
    events[idx].status = EventStatus::Snoozed;
    events[idx].snoozed_until = Some(date(2026, 4, 25));
    store.save_events(&events).unwrap();

    let reloaded = store.load_events().unwrap();
    assert_eq!(reloaded.len(), before_count, "snooze must not create a new event");
    let snoozed = reloaded.iter().find(|e| e.status == EventStatus::Snoozed).unwrap();
    assert_eq!(snoozed.snoozed_until, Some(date(2026, 4, 25)));
}

#[test]
fn skip_soil_wet_reschedules_sooner_than_a_full_cycle() {
    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    let bougainvillea = plants.iter().find(|p| p.id == "bougainvillea").unwrap();
    let today = date(2026, 4, 10);
    let full_cycle = next_water_due(today, bougainvillea);
    let recheck = skip_recheck_due(today, bougainvillea);
    assert!(recheck < full_cycle, "skip should recheck sooner than a full new cycle");
    assert!(recheck > today, "recheck must still be in the future");
}

// ---- Multi-day offline catch-up: no backlog spam ----

#[test]
fn offline_catch_up_does_not_chain_multiple_missed_cycles() {
    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    let plant = plants.iter().find(|p| p.id == "golden-pothos").unwrap();

    // Pretend the app was last open 30 days ago and has been closed since.
    let last_seen = date(2026, 3, 1);
    let reopened = date(2026, 3, 31);
    let stale_due = next_water_due(last_seen, plant); // e.g. last_seen + a few days

    // Regardless of how long ago stale_due was, resolving "today" always
    // computes the next date from today, never from stale_due -- so it can
    // never be more than one interval ahead of today.
    let interval = water_interval_days(plant.moisture_class, season_for_month(reopened.month()), plant.is_hanging);
    let next = next_due(reopened, plant, TaskType::Water);

    assert!(stale_due < reopened, "sanity check: the old due date really is in the past");
    assert_eq!(next, reopened + Duration::days(interval));
    assert!(
        (next - reopened).num_days() <= interval,
        "catch-up must not stack multiple missed intervals into one jump"
    );
}

// ---- Atomic write survives a stray leftover temp file ----

#[test]
fn first_run_starts_empty() {
    // A fresh install must not inherit anyone else's plants -- the user
    // adds their own from the catalog.
    let store = temp_store();
    store.ensure_initialised().unwrap();

    assert!(store.load_plants().unwrap().is_empty(), "new install should have no plants");
    assert!(store.load_events().unwrap().is_empty(), "new install should have no care events");
    assert_eq!(store.load_spaces().unwrap().len(), 1, "should start with one default space");
}

#[test]
fn atomic_write_round_trips_and_ignores_stray_tmp_files() {
    let store = temp_store();
    store.ensure_initialised().unwrap();

    let plants: Vec<_> = catalog::all().iter().take(3).map(|e| e.to_profile()).collect();
    store.save_plants(&plants).unwrap();
    assert_eq!(store.load_plants().unwrap().len(), 3);

    // A crash mid-write leaves a `plants.json.<uuid>.tmp` sibling behind.
    // Reads only ever open the exact `plants.json` name (never a glob), so
    // a stray tmp file -- even one holding different, valid JSON -- must not
    // change what gets loaded.
    let stray = store.dir().join(format!("plants.json.{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&stray, b"[]").unwrap();

    let reloaded = store.load_plants().unwrap();
    assert_eq!(reloaded.len(), 3, "stray .tmp file must not shadow the real file");
    assert_eq!(reloaded[0].id, plants[0].id);
}

// ---- Hemisphere awareness ----

#[test]
fn southern_hemisphere_seasons_are_six_months_out_of_phase() {
    use plant_health_tracker_lib::models::Season;
    use plant_health_tracker_lib::schedule::ScheduleContext;

    let december = date(2026, 12, 15);
    let north = ScheduleContext { latitude: Some(51.5), weather: &[] }; // London
    let south = ScheduleContext { latitude: Some(-33.9), weather: &[] }; // Sydney

    assert_eq!(north.season(december), Season::Mild);
    assert_eq!(
        south.season(december),
        Season::Monsoon,
        "December below the equator must not be treated as northern winter"
    );
    // With no latitude configured the behaviour is the original month-only model.
    assert_eq!(ScheduleContext::EMPTY.season(december), Season::Mild);
}

// ---- Weather adjustment ----

fn weather_day(d: NaiveDate, rain: f64, tmax: f64) -> plant_health_tracker_lib::models::DailyWeather {
    plant_health_tracker_lib::models::DailyWeather {
        date: d,
        precipitation_mm: rain,
        temp_max_c: tmax,
        temp_min_c: tmax - 8.0,
    }
}

#[test]
fn recent_rain_pushes_watering_out_and_heat_pulls_it_in() {
    use plant_health_tracker_lib::schedule::{weather_adjustment, ScheduleContext, WEATHER_LOOKBACK_DAYS};

    let today = date(2026, 4, 20);
    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    let plant = plants.iter().find(|p| p.id == "curry-leaf").unwrap();

    let dry = [weather_day(today, 0.0, 26.0), weather_day(today - Duration::days(1), 0.0, 25.0)];
    let soaked = [weather_day(today, 9.0, 24.0), weather_day(today - Duration::days(1), 6.0, 24.0)];
    let baking = [weather_day(today, 0.0, 39.0), weather_day(today - Duration::days(1), 0.0, 38.0)];

    assert_eq!(weather_adjustment(today, &dry, WEATHER_LOOKBACK_DAYS).extra_days, 0);
    assert_eq!(weather_adjustment(today, &soaked, WEATHER_LOOKBACK_DAYS).extra_days, 2);
    assert_eq!(weather_adjustment(today, &baking, WEATHER_LOOKBACK_DAYS).extra_days, -1);

    let base = next_water_due(today, plant);
    let after_rain = plant_health_tracker_lib::schedule::next_water_due_ctx(
        today,
        plant,
        ScheduleContext { latitude: Some(12.97), weather: &soaked },
    );
    let after_heat = plant_health_tracker_lib::schedule::next_water_due_ctx(
        today,
        plant,
        ScheduleContext { latitude: Some(12.97), weather: &baking },
    );
    assert!(after_rain > base, "rain should delay the next watering");
    assert!(after_heat < base, "a hot spell should bring it forward");
}

#[test]
fn weather_older_than_the_lookback_window_is_ignored() {
    use plant_health_tracker_lib::schedule::{weather_adjustment, WEATHER_LOOKBACK_DAYS};
    let today = date(2026, 4, 20);
    // A downpour a fortnight ago says nothing about today's soil.
    let ancient = [weather_day(today - Duration::days(14), 40.0, 25.0)];
    assert_eq!(weather_adjustment(today, &ancient, WEATHER_LOOKBACK_DAYS).extra_days, 0);
}

#[test]
fn watering_interval_never_collapses_below_one_day() {
    use plant_health_tracker_lib::schedule::{next_water_due_ctx, ScheduleContext};
    let today = date(2026, 4, 20);
    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    // Thirstiest class, hanging, in a heatwave -- the one case that could
    // otherwise land on "due again today" and immediately re-fire.
    let thirsty = plants
        .iter()
        .find(|p| p.is_hanging && matches!(p.moisture_class, plant_health_tracker_lib::models::MoistureClass::ConsistentlyMoist))
        .expect("catalog should contain a thirsty hanging plant");
    let baking = [weather_day(today, 0.0, 41.0)];
    let due = next_water_due_ctx(today, thirsty, ScheduleContext { latitude: Some(12.97), weather: &baking });
    assert!(due > today, "next watering must always be at least tomorrow");
}

#[test]
fn offline_with_no_weather_matches_the_original_month_only_schedule() {
    use plant_health_tracker_lib::schedule::{next_water_due_ctx, ScheduleContext};
    let today = date(2026, 8, 10);
    let plants: Vec<_> = catalog::all().iter().map(|e| e.to_profile()).collect();
    for plant in plants.iter().take(10) {
        assert_eq!(
            next_water_due_ctx(today, plant, ScheduleContext::EMPTY),
            next_water_due(today, plant),
            "empty context must behave exactly like the offline fallback"
        );
    }
}
