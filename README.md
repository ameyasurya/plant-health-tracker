# Plant Health Tracker

A small, frameless Windows desktop widget that keeps your plants alive. It
tracks watering and feeding, and adjusts the schedule to the season and to
your local weather — so a week of rain doesn't have it nagging you to water.

<!-- Screenshot goes here. Save it as docs/screenshot.png and uncomment:
![The widget on the desktop](docs/screenshot.png)
-->


## Why

Plant care apps are phone apps, and a phone app is easy to ignore. This
lives on the desktop as a small always-on card: if something needs water
today, it's already on screen. The mascot's mood mirrors the worst state in
the list — happy when nothing's due, wilting when something's overdue — so
it's glanceable without reading a single row.

## Install

1. Download the latest `.exe` installer from
   [Releases](../../releases) and run it.
2. **Windows will warn you the app is unsigned** — "Windows protected your
   PC". That's expected: a code-signing certificate costs money and this is
   a free hobby project. Click **More info** → **Run anyway**.
3. On first launch your plant list is empty. Press **+** to add plants, and
   open **Settings** to set your location.

Nothing is sent anywhere except the weather lookups described below, and
those are optional.

## Features

- **Add plants from a built-in catalog** of ~60 common houseplants, herbs,
  succulents and flowering plants. Search by common name, scientific name,
  or local name — "kadi patta", "money plant", "mother in law's tongue" all
  work. Picking one fills in its light, watering and feeding needs, so you
  don't have to know what a "moisture class" is. Anything not in the
  catalog can be added manually.
- **Weather-aware watering.** Recent rain pushes watering out, a hot spell
  pulls it forward. Seasons follow your hemisphere, and the day boundary
  follows your timezone.
- **Feeding schedules** per plant group (flowering/fruiting, citrus,
  foliage, herb/succulent), with a dormancy window where feeding pauses
  over the cool months and resumes automatically.
- **Catch-up rule.** Close the app for a week and you get one overdue item
  per plant, not a pile of stacked missed cycles. Marking something done
  reschedules from *when you actually did it*, so being late never drags
  the schedule permanently behind.
- **Spaces.** Group plants by where they live — balcony, kitchen, bedroom —
  and filter to one at a time.
- **Undo on every action.** Done / snooze / skip hold for a few seconds
  before they're committed, so a mis-click costs nothing.
- **Plant details** with uses, cultural background and a fun fact.
- **All-clear state** that shows a fact about a plant you actually own,
  instead of an empty list.
- Daily digest notification (one per day, not one per plant), tray icon,
  pin-on-top, minimise-to-tray, single-instance guard, light/dark themes,
  and a resizable window that stays usable down to two visible rows.

## Privacy and network use

The app is offline-first. It has no account, no telemetry and no cloud
sync; your data is plain JSON on your own machine.

Two optional network calls exist, both free and keyless:

| What | Service | When |
|---|---|---|
| City search | [Open-Meteo geocoding](https://open-meteo.com/en/docs/geocoding-api) | Only while you type in the location search |
| Forecast | [Open-Meteo](https://open-meteo.com/) | Every ~6 hours, if a location is set |
| IP location | [ipwho.is](https://ipwho.is/) | **Only** when you press "Detect" |

IP detection is never automatic — it sends your IP to a third party, so it
sits behind an explicit button. Turning off *"Use weather to adjust
watering"* in Settings stops all network activity entirely; the schedule
then falls back to month-based seasons.

Data lives in `%APPDATA%\com.ameya.plant-health-tracker\` as `plants.json`,
`care-log.json`, `spaces.json`, `settings.json` and `weather.json`.

## Building from source

Prerequisites:

1. **Rust** — via [rustup.rs](https://rustup.rs). Restart your terminal after.
2. **Node.js** 18+.
3. **Microsoft C++ Build Tools** with the *Desktop development with C++*
   workload — Tauri needs the MSVC linker. See
   [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```powershell
npm install
npm run tauri dev
```

To produce installers (MSI + NSIS) under `src-tauri/target/release/bundle/`:

```powershell
npm run tauri build
```

### UI preview without the desktop shell

`preview.html` runs the real React UI in a browser tab against a mocked
Tauri IPC layer, which is much faster for styling work than relaunching the
native window:

```powershell
npm run dev
```

then open `http://localhost:1420/preview.html`. Dev-only; not part of the
production build.

### Tests

```powershell
cd src-tauri
cargo test
```

Covers schedule generation across every catalog species and season,
month-boundary transitions, the feeding dormancy window, hemisphere
inversion, weather adjustment (including that stale weather is ignored and
intervals never collapse below a day), the offline fallback matching the
month-only schedule exactly, done/snooze/skip recomputation, multi-day
catch-up, and atomic writes surviving a stray temp file.

## Known gaps

- **Icons are placeholders.** `src-tauri/icons/` holds simple generated
  icons; `tray-icon.png` is present but unreferenced, so the tray falls
  back to the default window icon.
- **Windows only.** Nothing is deliberately platform-locked apart from the
  bundle targets and the autostart path, but it has only been built and run
  on Windows.
- **The installer is unsigned**, so SmartScreen warns on first run.
- **Catalog care values are general guidance**, not a substitute for
  watching your own plants. Two entries carried over from the original
  inventory (Ixora's watering class, Indian Borage's light class) were
  inferred and are flagged with a `*`.

## Licence

MIT — see [LICENSE](LICENSE).
