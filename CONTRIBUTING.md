# Working on Plant Health Tracker

Notes for anyone building or modifying the app. Nothing here is needed just
to use it.

## Prerequisites

1. **Rust**, via [rustup.rs](https://rustup.rs). Restart your terminal after.
2. **Node.js** 18+.
3. A C toolchain. On Windows that is the **Microsoft C++ Build Tools** with
   the *Desktop development with C++* workload, which provides the MSVC
   linker Tauri needs. On macOS it is the **Xcode Command Line Tools**
   (`xcode-select --install`). See
   [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tauri dev
```

## Building the installer

```sh
npm run tauri build
```

Output lands in `src-tauri/target/release/bundle/nsis/` on Windows and
`src-tauri/target/release/bundle/dmg/` on macOS.

A plain macOS build only targets the architecture you are on, so an Apple
Silicon machine produces something an Intel Mac cannot run. Releases use the
universal target instead, which covers both in one download:

```sh
npm run tauri build -- --target universal-apple-darwin
```

That needs both architectures installed
(`rustup target add aarch64-apple-darwin x86_64-apple-darwin`) and writes to
`src-tauri/target/universal-apple-darwin/release/bundle/dmg/`.

Use those rather than a bare `cargo build --release`. Plain cargo omits the
`custom-protocol` feature, so the binary builds and launches happily but still
expects the Vite dev server, showing "localhost refused to connect" instead of
the widget.

## Layout

- `src/` — React + TypeScript frontend.
- `src-tauri/src/` — Rust backend: schedule engine, JSON persistence, reminder
  loop, tray, weather and location lookups.
- `src-tauri/data/catalog.json` — the bundled species catalog, compiled into
  the binary.
- `src-tauri/tauri.macos.conf.json` — macOS-only config, merged over
  `tauri.conf.json` automatically when building for macOS. See below.

There is no database. Three JSON files are written atomically (temp file plus
rename), which is plenty for a single-user widget and keeps everything
readable in a text editor.

## macOS specifics

`tauri.macos.conf.json` carries three things that only matter on macOS:

- **`macOSPrivateApi`**, together with the matching `macos-private-api`
  Cargo feature declared in a `cfg(target_os = "macos")` dependency block.
  The widget is a transparent, undecorated window, and macOS ignores
  transparency unless both halves of that switch are on. Miss either one
  and the window renders with an opaque system background instead of the
  rounded card, with no build error to explain why.
- **`dmg`** as the bundle target, replacing `nsis`.
- **Ad-hoc code signing** (`"signingIdentity": "-"`). Apple Silicon refuses
  to execute an unsigned binary at all, and the universal build runs `lipo`,
  which invalidates whatever signature the linker applied. Hardened runtime
  is switched off because it is only needed for notarisation, and
  notarisation needs a paid Apple Developer account. Without it, Gatekeeper
  blocks the first launch and users have to allow the app once under
  System Settings, Privacy & Security.

## Tests

```sh
cd src-tauri
cargo test
```

Covers schedule generation across every catalog species and season,
month-boundary transitions, the feeding dormancy window, hemisphere inversion,
weather adjustment (including that stale weather is ignored and intervals never
collapse below a day), the offline fallback matching the month-only schedule
exactly, done/snooze/skip recomputation, multi-day catch-up, and atomic writes
surviving a stray temp file.

## UI preview without the desktop shell

`preview.html` runs the real React UI in an ordinary browser tab with the Tauri
IPC layer replaced by mocks:

```sh
npm run dev
```

then open `/preview.html` on the URL Vite prints, by default
<http://localhost:1420/preview.html>. The port is pinned in `vite.config.ts`
with `strictPort` because `tauri.conf.json` points `devUrl` at it, so Vite
fails rather than quietly moving.

The native window hot-reloads frontend edits too, so this is not about
iteration speed. It is useful because it needs no Rust toolchain, cannot touch
real plant data, and can fake states that are awkward to reach for real such as
overdue plants or an empty list.

It is dev-only: Vite builds `index.html` alone, so neither `preview.html` nor
its mock code ends up in the shipped bundle.

## Icons

`app-icon.svg` is the source artwork, built from the same shapes as
`src/components/Mascot.tsx` so the icon and the in-app character cannot drift
apart. The pot is omitted because the whole plant is unreadable at tray size.

```sh
npx tauri icon app-icon.svg
```

That also emits iOS, Android and Windows Store variants. Only the five files
listed under `bundle.icon` in `tauri.conf.json` are used, `icon.icns` among
them because macOS bundling requires it, so the rest can be deleted.

One trap worth knowing: `tauri-build` embeds the icon into a Windows resource
from `build.rs`, and only declares `rerun-if-changed` for `tauri.conf.json` and
the capabilities directory. Left alone, editing an icon would not rebuild the
resource and the binary would keep shipping the previous one, which is easy to
miss in CI where `target/` is restored from cache. `build.rs` therefore
watches `icons/` itself.

## Releasing

Version lives in three files that must agree: `package.json`,
`src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`.

Pushing a `v*` tag runs `.github/workflows/release.yml`, which type-checks,
runs the tests and builds an installer on a Windows runner and a macOS one,
then attaches both to a single **draft** release. Publishing it is a manual
step on GitHub.

Neither installer is signed by an authority the OS recognises, so Windows
SmartScreen warns on first run and macOS blocks the first launch until the
user allows the app under Privacy & Security. A code-signing certificate and
an Apple Developer account are the only fixes, and both cost money.
