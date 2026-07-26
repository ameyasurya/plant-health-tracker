# Plant Health Tracker

A Windows desktop widget that helps you never miss watering or feeding again. It
tracks watering and feeding, and adjusts the schedule to the season and to
your local weather, so a week of rain doesn't have it nagging you to water.

![Three copies of the widget on a desktop: one listing overdue plants with an unhappy mascot, one all-clear showing a plant fun fact, and one for a second space](docs/screenshot.png)


## Why

Plant care apps are phone apps, and a phone app is easy to ignore. This
lives on the desktop as a small always-on card: if something needs water
today, it's already on screen. The mascot's mood mirrors the worst state in
the list: happy when nothing's due, wilting when something's overdue, so
it's glanceable without reading a single row.

## Install

1. Download the latest `.exe` installer from
   [Releases](../../releases) and run it.
2. **Windows will warn you the app is unsigned**, showing "Windows protected your
   PC". That's expected: a code-signing certificate costs money and this is
   a free hobby project. Click **More info** → **Run anyway**.
3. On first launch your plant list is empty. Press **+** to add plants, and
   open **Settings** to set your location.

Nothing is sent anywhere except the optional location and weather lookups
described below.

## Features

- **Add plants from a built-in catalog** of ~150 common houseplants, herbs,
  succulents, cacti, flowering and fruiting plants. Search by common name,
  scientific name, or local name. "kadi patta", "money plant", "mother in
  law's tongue" all work. Picking one fills in its light, watering and
  feeding needs, so you don't have to know what a "moisture class" is.
  Anything not in the catalog can be added manually.
- **Tell it when you last watered and fed**, when you add a plant. Without
  that the first reminder is a guess, and it can only guess wrong in one
  direction or the other. "Not sure" is always an option.
- **Weather-aware watering.** Recent rain pushes watering out, a hot spell
  pulls it forward. Seasons follow your hemisphere, and the day boundary
  follows your timezone.
- **Feeding schedules** per plant group (flowering/fruiting, citrus,
  foliage, herb/succulent), with a dormancy window where feeding pauses
  over the cool months and resumes automatically.
- **Log care you did off-schedule.** Fed a plant three weeks before it was
  due? Open it and say so, and the next feed is worked out from when you
  actually did it. Watering intervals are short enough that the reminder
  usually catches them, but feeding cadences run three to seven weeks, so
  without this the app would keep counting down to a date it had no way of
  knowing was wrong.
- **Catch-up rule.** Close the app for a week and you get one overdue item
  per plant, not a pile of stacked missed cycles. Marking something done
  reschedules from *when you actually did it*, so being late never drags
  the schedule permanently behind.
- **Spaces.** Group plants by where they live (balcony, kitchen,
  bedroom) and filter to one at a time.
- **Overview tab** listing every plant under its space, with its watering
  and feeding state side by side, so nothing quietly falls behind.
- **A to-do list**, because the widget is already sitting on your desktop.
  Unfinished items carry over and are marked with their age rather than
  being cleared overnight.
- **Undo on every action.** Done / snooze / skip hold for a few seconds
  before they're committed, so a mis-click costs nothing.
- **Plant details** with uses, cultural background and a fun fact.
- **All-clear state** that shows a fact about a plant you actually own,
  instead of an empty list.
- Daily digest notification (one per day, not one per plant), taskbar button
  and tray icon, pin-on-top (also toggleable from the tray), hide-to-tray,
  single-instance guard, light/dark themes, and a resizable window that stays
  usable down to two visible rows.

## Privacy and network use

The app is offline-first. It has no account, no telemetry and no cloud
sync; your data is plain JSON on your own machine.

Three network calls exist, all optional, and all free without an API key
or account:

| What | Service | When |
|---|---|---|
| Forecast | [Open-Meteo](https://open-meteo.com/) | Every ~6 hours, only if a location is set |
| City search | [Open-Meteo geocoding](https://open-meteo.com/en/docs/geocoding-api) | Only while you type in the location search |
| IP location | [ipwho.is](https://ipwho.is/) | **Only** when you press "Detect" |

IP detection is never automatic, because it sends your IP address to a third
party, so it sits behind a button you have to press.

Turning off *"Use weather to adjust watering"* in Settings stops the app
making any network request on its own, and the schedule falls back to
month-based seasons. The city search and "Detect" button still reach out
when you actively use them.

Data lives in `%APPDATA%\com.ameya.plant-health-tracker\` as `plants.json`,
`care-log.json`, `spaces.json`, `settings.json`, `weather.json` and
`todos.json`.

## Building from source

You need Rust, Node 18+ and the Microsoft C++ Build Tools:

```powershell
npm install
npm run tauri dev
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for building the installer, running the
tests, regenerating icons and cutting a release.

## Known gaps

- **Windows only.** Nothing is deliberately platform-locked apart from the
  bundle targets and the autostart path, but it has only been built and run
  on Windows.
- **The installer is unsigned**, so SmartScreen warns on first run.
- **Catalog care values are general guidance**, not a substitute for
  watching your own plants. A species entry is a sensible starting point;
  edit any plant if your conditions differ.
- **No plant photos.** Rows and details are text only.

## Licence

MIT. See [LICENSE](LICENSE).
