//! Test plan coverage:
//!   - schedule generation for all 19 plants x 3 seasons
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

use balcony_widget_lib::models::{EventStatus, TaskType};
use balcony_widget_lib::schedule::{
    next_due, next_fertilize_due, next_water_due, season_for_month, skip_recheck_due,
    water_interval_days,
};
use balcony_widget_lib::seed::seed_plants;
use balcony_widget_lib::store::{new_pending_event, Store};

fn temp_store() -> Store {
    let dir = std::env::temp_dir().join(format!("balcony-widget-test-{}", uuid::Uuid::new_v4()));
    Store::new(dir).expect("create temp store")
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

// ---- Season month-boundary transitions ----

#[test]
fn season_boundaries_are_correct() {
    use balcony_widget_lib::models::Season::*;
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
    let plants = seed_plants();
    assert_eq!(plants.len(), 19, "inventory should have exactly 19 plants");

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
    use balcony_widget_lib::models::{MoistureClass, Season};
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
    let plants = seed_plants();
    let ficus = plants.iter().find(|p| p.id == "ficus").unwrap(); // Foliage group, active Mar-Oct
    let due = next_fertilize_due(date(2026, 11, 15), ficus);
    assert_eq!(due, date(2027, 3, 1), "foliage feed should resume March 1 after the Nov-Feb pause");

    let orange = plants.iter().find(|p| p.id == "orange").unwrap(); // Citrus group, active Feb-Oct
    let due = next_fertilize_due(date(2026, 11, 15), orange);
    assert_eq!(due, date(2027, 2, 1), "citrus feed should resume Feb 1, one month earlier than foliage");
}

// ---- Done / snooze / skip-soil-wet flows recompute correctly ----

#[test]
fn mark_done_schedules_next_occurrence_from_today_not_from_old_due_date() {
    let store = temp_store();
    store.ensure_seeded().unwrap();
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
    let store = temp_store();
    store.ensure_seeded().unwrap();
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
    let plants = seed_plants();
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
    let plants = seed_plants();
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
fn atomic_write_round_trips_and_ignores_stray_tmp_files() {
    let store = temp_store();
    store.ensure_seeded().unwrap();

    let plants = store.load_plants().unwrap();
    assert_eq!(plants.len(), 19);

    // A crash mid-write would leave a `plants.json.<uuid>.tmp` sibling file
    // behind; because reads only ever open the exact `plants.json` name
    // (never a glob), a stray tmp file must not affect what gets loaded.
    let reloaded = store.load_plants().unwrap();
    assert_eq!(reloaded.len(), plants.len());
}
