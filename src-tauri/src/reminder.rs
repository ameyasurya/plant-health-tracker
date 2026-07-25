//! Background reminder engine.
//!
//! Fires ONE daily digest notification (e.g. "3 tasks due: Bougainvillea,
//! Jasmine, Citrus") at the user's configured time, rather than a
//! notification per plant/task -- per-task notifications would spam given
//! up to 19 plants x 2 task types. Clicking the notification focuses the
//! window, which already opens on the Due Today tab.
//!
//! Catch-up: if the app was closed for several days, overdue events keep
//! their original due date (see commands::effective_due / the catch-up
//! rule in mark_done) -- so on relaunch this loop sees "N due" once, not
//! a backlog of one notification per missed day.

use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::commands::AppState;
use crate::models::EventStatus;
use crate::time::{now_local_time, parse_hhmm, today_local};

const POLL_INTERVAL: Duration = Duration::from_secs(60);

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(err) = tick(&app) {
                eprintln!("reminder tick failed: {err}");
            }
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    });
}

fn tick(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let store = state.store.lock().map_err(|e| e.to_string())?;
    let mut settings = store.load_settings().map_err(|e| e.to_string())?;
    // Digest timing follows the user's own location, not a fixed zone.
    let location = settings.location.clone();
    let today = today_local(location.as_ref());

    if settings.last_digest_sent_on == Some(today) {
        return Ok(()); // already sent today
    }
    let Some(target_time) = parse_hhmm(&settings.notification_time) else {
        return Ok(());
    };
    if now_local_time(location.as_ref()) < target_time {
        return Ok(()); // not time yet
    }

    let plants = store.load_plants().map_err(|e| e.to_string())?;
    let events = store.load_events().map_err(|e| e.to_string())?;

    let mut names: Vec<String> = Vec::new();
    for event in events.iter().filter(|e| {
        matches!(e.status, EventStatus::Pending | EventStatus::Snoozed)
    }) {
        let due = event.snoozed_until.unwrap_or(event.due_at);
        if due <= today {
            if let Some(plant) = plants.iter().find(|p| p.id == event.plant_id) {
                if !names.contains(&plant.common_name) {
                    names.push(plant.common_name.clone());
                }
            }
        }
    }

    if !names.is_empty() {
        let body = if names.len() <= 4 {
            names.join(", ")
        } else {
            format!("{}, +{} more", names[..4].join(", "), names.len() - 4)
        };
        let title = format!(
            "{} task{} due",
            names.len(),
            if names.len() == 1 { "" } else { "s" }
        );
        let _ = app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show();
    }

    settings.last_digest_sent_on = Some(today);
    store.save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(())
}
