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

- **Add plants from a built-in catalog** of ~60 common houseplants, herbs,
  succulents and flowering plants. Search by common name, scientific name,
  or local name. "kadi patta", "money plant", "mother in law's tongue" all
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
- **Spaces.** Group plants by where they live (balcony, kitchen,
  bedroom) and filter to one at a time.
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
`care-log.json`, `spaces.json`, `settings.json` and `weather.json`.

## Building from source

Prerequisites:

1. **Rust**, via [rustup.rs](https://rustup.rs). Restart your terminal after.
2. **Node.js** 18+.
3. **Microsoft C++ Build Tools** with the *Desktop development with C++*
   workload, which provides the MSVC linker Tauri needs. See
   [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```powershell
npm install
npm run tauri dev
```

To produce the installer under `src-tauri/target/release/bundle/nsis/`:

```powershell
npm run tauri build
```

Use that rather than a bare `cargo build --release`. Plain cargo omits the
`custom-protocol` feature, so the binary looks fine but still expects the
Vite dev server: launched on its own it shows "localhost refused to
connect" instead of the widget.

### Icons

`app-icon.svg` is the source artwork: the mascot's winking flower head,
using the same shapes as the in-app character so the two can't drift
apart. The pot is left out because it turns to mush at tray size. To
regenerate the PNG and ICO set after editing it:

```powershell
npx tauri icon app-icon.svg
```

It also emits iOS, Android, macOS and Windows Store variants. Only the four
files listed in `tauri.conf.json`'s `bundle.icon` are needed, so the rest can
be deleted.

The icon is embedded into a Windows resource by `build.rs`, and `tauri-build`
only declares `rerun-if-changed` for `tauri.conf.json` and the capabilities
directory. Left alone, that means editing an icon does not rebuild the
resource and the binary keeps shipping the previous one, which is especially
easy to miss in CI where `target/` is restored from cache. `build.rs`
therefore watches `icons/` itself, so a regenerated icon rebuilds normally.

### UI preview without the desktop shell

`preview.html` runs the real React UI in an ordinary browser tab, with the
Tauri IPC layer replaced by mocks:

```powershell
npm run dev
```

then open the `/preview.html` path on the URL Vite prints, by default
<http://localhost:1420/preview.html>. That port is pinned in
`vite.config.ts` with `strictPort`, because `tauri.conf.json` points
`devUrl` at it; Vite fails rather than quietly moving to another port.

The native window hot-reloads frontend edits too, so this isn't about
iteration speed. It's useful because it:

- **needs no Rust toolchain.** `npm run dev` is plain Vite, so UI work
  doesn't require MSVC build tools or a compiled backend;
- **can't touch your real data**, since every command is mocked, which
  makes it safe to try destructive paths like deleting plants;
- **fakes states that are awkward to reach for real**, such as overdue
  plants or an empty list;
- gives you normal browser devtools and a resizable frame for checking
  layout at the window's minimum size.

It is dev-only: Vite builds `index.html` alone, so neither `preview.html`
nor its mock code is in the shipped bundle.

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
