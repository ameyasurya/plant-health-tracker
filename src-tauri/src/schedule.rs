//! Season-adjusted schedule logic for Bengaluru balcony conditions.
//!
//! Everything here is a pure function of (today, plant profile) so it is
//! trivial to unit test across all 19 plants and all 3 seasons without
//! touching the filesystem or the clock. All dates are calendar dates in
//! Asia/Kolkata -- see time::today_ist() for the one place "now" enters
//! the picture.

use chrono::{Datelike, Duration, NaiveDate};

use crate::models::{FertilizeGroup, MoistureClass, PlantProfile, Season, TaskType};

pub fn season_for_month(month: u32) -> Season {
    match month {
        3..=5 => Season::HotDry,
        6..=10 => Season::Monsoon,
        _ => Season::Mild, // 11, 12, 1, 2
    }
}

/// (shorter interval for hanging plants, longer interval for potted/non-hanging).
/// Hanging plants dry faster and get the shorter end of the range.
fn water_interval_range(moisture: MoistureClass, season: Season) -> (i64, i64) {
    use MoistureClass::*;
    use Season::*;
    match (moisture, season) {
        (ConsistentlyMoist, HotDry) => (1, 1),
        (ConsistentlyMoist, Monsoon) => (1, 2),
        (ConsistentlyMoist, Mild) => (2, 3),
        (Moderate, HotDry) => (2, 2),
        (Moderate, Monsoon) => (2, 3),
        (Moderate, Mild) => (3, 4),
        (Drier, HotDry) => (3, 3),
        (Drier, Monsoon) => (4, 5),
        (Drier, Mild) => (5, 7),
    }
}

pub fn water_interval_days(moisture: MoistureClass, season: Season, is_hanging: bool) -> i64 {
    let (short, long) = water_interval_range(moisture, season);
    if is_hanging {
        short
    } else {
        long
    }
}

pub fn soil_check_instruction(moisture: MoistureClass) -> &'static str {
    match moisture {
        MoistureClass::ConsistentlyMoist => "Keep consistently moist",
        MoistureClass::Moderate => "Check top 2-3cm, water if dry",
        MoistureClass::Drier => "Let soil dry between waterings",
    }
}

pub struct FertilizePlan {
    pub cadence_days: i64,
    /// 1-indexed months (1 = January) this plant is fed. Outside this
    /// window fertilizing is paused, per RHS/UF-IFAS guidance for
    /// dormancy in Bengaluru's mild season.
    pub active_months: &'static [u32],
    pub fertilizer_type: &'static str,
}

pub fn fertilize_plan(group: FertilizeGroup) -> FertilizePlan {
    match group {
        FertilizeGroup::FloweringFruiting => FertilizePlan {
            cadence_days: 21,
            active_months: &[3, 4, 5, 6, 7, 8, 9, 10],
            fertilizer_type: "Bloom / high-potassium feed",
        },
        FertilizeGroup::Citrus => FertilizePlan {
            cadence_days: 42,
            active_months: &[2, 3, 4, 5, 6, 7, 8, 9, 10],
            fertilizer_type: "Citrus feed, light dose",
        },
        FertilizeGroup::Foliage => FertilizePlan {
            cadence_days: 30,
            active_months: &[3, 4, 5, 6, 7, 8, 9, 10],
            fertilizer_type: "Balanced foliage feed",
        },
        FertilizeGroup::HerbSucculent => FertilizePlan {
            cadence_days: 49,
            active_months: &[3, 4, 5, 6, 7, 8, 9, 10],
            fertilizer_type: "Light balanced feed",
        },
    }
}

/// Always "feed only when soil is already moist" per the plan, regardless
/// of fertilizer type, to avoid root burn.
pub fn fertilize_instruction(group: FertilizeGroup) -> String {
    format!("{} · soil moist only", fertilize_plan(group).fertilizer_type)
}

fn first_day_of_next_active_month(from: NaiveDate, active_months: &[u32]) -> NaiveDate {
    let mut year = from.year();
    let mut month = from.month();
    for _ in 0..13 {
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
        if active_months.contains(&month) {
            return NaiveDate::from_ymd_opt(year, month, 1).expect("valid calendar date");
        }
    }
    // Unreachable given active_months is always non-empty, but keep a safe fallback.
    from
}

pub fn next_water_due(today: NaiveDate, plant: &PlantProfile) -> NaiveDate {
    let season = season_for_month(today.month());
    let interval = water_interval_days(plant.moisture_class, season, plant.is_hanging);
    today + Duration::days(interval)
}

pub fn next_fertilize_due(today: NaiveDate, plant: &PlantProfile) -> NaiveDate {
    let plan = fertilize_plan(plant.fertilize_group);
    if !plan.active_months.contains(&today.month()) {
        return first_day_of_next_active_month(today, plan.active_months);
    }
    let candidate = today + Duration::days(plan.cadence_days);
    if plan.active_months.contains(&candidate.month()) {
        candidate
    } else {
        first_day_of_next_active_month(candidate, plan.active_months)
    }
}

pub fn next_due(today: NaiveDate, plant: &PlantProfile, task_type: TaskType) -> NaiveDate {
    match task_type {
        TaskType::Water => next_water_due(today, plant),
        TaskType::Fertilize => next_fertilize_due(today, plant),
    }
}

/// Short recheck window used when the user skips a watering because the
/// soil is visibly still wet -- not a full new cycle, just "look again
/// soon", capped so it never exceeds the plant's normal interval.
pub fn skip_recheck_due(today: NaiveDate, plant: &PlantProfile) -> NaiveDate {
    let season = season_for_month(today.month());
    let full_interval = water_interval_days(plant.moisture_class, season, plant.is_hanging);
    let recheck = std::cmp::max(1, full_interval / 2);
    today + Duration::days(recheck)
}

pub fn instruction_for(task_type: TaskType, plant: &PlantProfile) -> String {
    match task_type {
        TaskType::Water => soil_check_instruction(plant.moisture_class).to_string(),
        TaskType::Fertilize => fertilize_instruction(plant.fertilize_group),
    }
}
