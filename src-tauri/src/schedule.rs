//! Season- and weather-adjusted schedule logic.
//!
//! Everything here is a pure function of its inputs -- (today, plant, and
//! optionally recent weather) -- so it can be unit tested across every
//! catalog species and season without touching the filesystem, the
//! network or the clock.
//!
//! Weather is optional throughout. With no location configured or the
//! network down, intervals fall back to the month-based season model, so
//! the widget keeps working offline.

use chrono::{Datelike, Duration, NaiveDate};

use crate::models::{DailyWeather, FertilizeGroup, MoistureClass, PlantProfile, Season, TaskType};

pub fn season_for_month(month: u32) -> Season {
    match month {
        3..=5 => Season::HotDry,
        6..=10 => Season::Monsoon,
        _ => Season::Mild, // 11, 12, 1, 2
    }
}

/// How recent rain and heat bend the base watering interval.
///
/// Deliberately conservative: this nudges an interval, it does not invent
/// a schedule. Soil in a pot on a covered balcony does not necessarily see
/// the rain the forecast reports, so the adjustment is capped and the
/// user's own "skip, soil still wet" action remains the real authority.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeatherAdjustment {
    pub extra_days: i64,
    pub reason: Option<&'static str>,
}

impl WeatherAdjustment {
    pub const NONE: Self = Self { extra_days: 0, reason: None };
}

/// Rain over this many mm in the recent window counts as a real soaking.
const SOAKING_MM: f64 = 10.0;
/// Lighter rain still buys a little time.
const DAMP_MM: f64 = 3.0;
/// Above this the plant is losing water fast enough to pull watering in.
const HEATWAVE_C: f64 = 34.0;

/// Looks at the days immediately before and including `today`.
pub fn weather_adjustment(today: NaiveDate, days: &[DailyWeather], lookback: i64) -> WeatherAdjustment {
    if days.is_empty() {
        return WeatherAdjustment::NONE;
    }
    let from = today - Duration::days(lookback);
    let recent: Vec<&DailyWeather> = days.iter().filter(|d| d.date > from && d.date <= today).collect();
    if recent.is_empty() {
        return WeatherAdjustment::NONE;
    }

    let rain: f64 = recent.iter().map(|d| d.precipitation_mm).sum();
    let hottest = recent
        .iter()
        .map(|d| d.temp_max_c)
        .filter(|t| !t.is_nan())
        .fold(f64::NEG_INFINITY, f64::max);

    if rain >= SOAKING_MM {
        return WeatherAdjustment { extra_days: 2, reason: Some("recent rain") };
    }
    if rain >= DAMP_MM {
        return WeatherAdjustment { extra_days: 1, reason: Some("recent rain") };
    }
    if hottest.is_finite() && hottest >= HEATWAVE_C {
        return WeatherAdjustment { extra_days: -1, reason: Some("hot spell") };
    }
    WeatherAdjustment::NONE
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

/// How many days back the rain/heat check looks.
pub const WEATHER_LOOKBACK_DAYS: i64 = 3;

/// Everything the schedule needs beyond the plant itself: where the user
/// is (for hemisphere) and what the weather has been doing.
///
/// `EMPTY` is the offline/unconfigured case and behaves exactly like the
/// original month-only logic, which is what keeps this safe to fall back
/// to when there is no location or no network.
#[derive(Debug, Clone, Copy)]
pub struct ScheduleContext<'a> {
    pub latitude: Option<f64>,
    pub weather: &'a [DailyWeather],
}

impl ScheduleContext<'_> {
    pub const EMPTY: ScheduleContext<'static> = ScheduleContext { latitude: None, weather: &[] };

    /// Season at the user's latitude -- below the equator the month is
    /// shifted six months so summer isn't treated as winter.
    pub fn season(&self, today: NaiveDate) -> Season {
        season_for_month(crate::time::season_month_for(self.latitude, today.month()))
    }

    fn effective_month(&self, date: NaiveDate) -> u32 {
        crate::time::season_month_for(self.latitude, date.month())
    }
}

pub fn next_water_due(today: NaiveDate, plant: &PlantProfile) -> NaiveDate {
    next_water_due_ctx(today, plant, ScheduleContext::EMPTY)
}

pub fn next_water_due_ctx(today: NaiveDate, plant: &PlantProfile, ctx: ScheduleContext) -> NaiveDate {
    let season = ctx.season(today);
    let base = water_interval_days(plant.moisture_class, season, plant.is_hanging);
    let adj = weather_adjustment(today, ctx.weather, WEATHER_LOOKBACK_DAYS);
    // Never below 1: even in a heatwave "water again today" would just
    // re-fire the reminder that was completed moments ago.
    let interval = (base + adj.extra_days).max(1);
    today + Duration::days(interval)
}

pub fn next_fertilize_due(today: NaiveDate, plant: &PlantProfile) -> NaiveDate {
    next_fertilize_due_ctx(today, plant, ScheduleContext::EMPTY)
}

pub fn next_fertilize_due_ctx(today: NaiveDate, plant: &PlantProfile, ctx: ScheduleContext) -> NaiveDate {
    let plan = fertilize_plan(plant.fertilize_group);
    if !plan.active_months.contains(&ctx.effective_month(today)) {
        return next_active_month_start(today, plan.active_months, ctx);
    }
    let candidate = today + Duration::days(plan.cadence_days);
    if plan.active_months.contains(&ctx.effective_month(candidate)) {
        candidate
    } else {
        next_active_month_start(candidate, plan.active_months, ctx)
    }
}

/// Walks forward to the first calendar month whose *effective* (hemisphere
/// adjusted) month is in the feeding window.
fn next_active_month_start(from: NaiveDate, active_months: &[u32], ctx: ScheduleContext) -> NaiveDate {
    let mut year = from.year();
    let mut month = from.month();
    for _ in 0..13 {
        month += 1;
        if month > 12 {
            month = 1;
            year += 1;
        }
        let candidate = NaiveDate::from_ymd_opt(year, month, 1).expect("valid calendar date");
        if active_months.contains(&ctx.effective_month(candidate)) {
            return candidate;
        }
    }
    from
}

pub fn next_due(today: NaiveDate, plant: &PlantProfile, task_type: TaskType) -> NaiveDate {
    next_due_ctx(today, plant, task_type, ScheduleContext::EMPTY)
}

pub fn next_due_ctx(
    today: NaiveDate,
    plant: &PlantProfile,
    task_type: TaskType,
    ctx: ScheduleContext,
) -> NaiveDate {
    match task_type {
        TaskType::Water => next_water_due_ctx(today, plant, ctx),
        TaskType::Fertilize => next_fertilize_due_ctx(today, plant, ctx),
    }
}

/// Short recheck window used when the user skips a watering because the
/// soil is visibly still wet -- not a full new cycle, just "look again
/// soon", capped so it never exceeds the plant's normal interval.
pub fn skip_recheck_due(today: NaiveDate, plant: &PlantProfile) -> NaiveDate {
    skip_recheck_due_ctx(today, plant, ScheduleContext::EMPTY)
}

pub fn skip_recheck_due_ctx(today: NaiveDate, plant: &PlantProfile, ctx: ScheduleContext) -> NaiveDate {
    let season = ctx.season(today);
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
