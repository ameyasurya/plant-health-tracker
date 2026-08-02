# Working on Plant Health Tracker

Notes for anyone building or modifying the app. Nothing here is needed just
to use it.

## Prerequisites

1. **Rust**, via [rustup.rs](https://rustup.rs). Restart your terminal after.
2. **Node.js** 18+.
3. **Microsoft C++ Build Tools** with the *Desktop development with C++*
   workload, which provides the MSVC linker Tauri needs. See
   [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

```powershell
npm install
npm run tauri dev
```

## Building the installer

```powershell
npm run tauri build
```

Output lands in `src-tauri/target/release/bundle/nsis/`.

Use that rather than a bare `cargo build --release`. Plain cargo omits the
`custom-protocol` feature, so the binary builds and launches happily but still
expects the Vite dev server, showing "localhost refused to connect" instead of
the widget.

## Layout

- `src/` — React + TypeScript frontend.
- `src-tauri/src/` — Rust backend: schedule engine, JSON persistence, reminder
  loop, tray, weather and location lookups.
- `src-tauri/data/catalog.json` — the bundled species catalog, compiled into
  the binary.

There is no database. Each JSON file is written atomically (temp file plus
rename), which is plenty for a single-user widget and keeps everything
readable in a text editor.

Anything added to a stored struct needs a serde default. Without one, an
older file on disk fails to parse as a whole, which would silently discard
the user's care history rather than erroring loudly.

## Tests

```powershell
cd src-tauri
cargo test
```

Covers schedule generation across every catalog species and season,
month-boundary transitions, the feeding dormancy window, hemisphere inversion,
weather adjustment (including that stale weather is ignored and intervals never
collapse below a day), the offline fallback matching the month-only schedule
exactly, done/snooze/skip recomputation, multi-day catch-up, anchoring a new
plant's first due dates to its stated care history, and atomic writes surviving
a stray temp file.

The catalog has its own checks in `src/catalog.rs`: unique ids, no duplicate
common or scientific names, and no empty copy fields. It is hand-authored, so
the realistic mistake is adding a species that is already there under a
slightly different name.

## UI preview without the desktop shell

`preview.html` runs the real React UI in an ordinary browser tab with the Tauri
IPC layer replaced by mocks:

```powershell
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

```powershell
npx tauri icon app-icon.svg
```

That also emits iOS, Android, macOS and Windows Store variants. Only the four
files listed under `bundle.icon` in `tauri.conf.json` are used, so the rest can
be deleted.

One trap worth knowing: `tauri-build` embeds the icon into a Windows resource
from `build.rs`, and only declares `rerun-if-changed` for `tauri.conf.json` and
the capabilities directory. Left alone, editing an icon would not rebuild the
resource and the binary would keep shipping the previous one — easy to miss in
CI, where `target/` is restored from cache. `build.rs` therefore watches
`icons/` itself.

## Releasing

Version lives in three files that must agree: `package.json`,
`src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`.

Pushing a `v*` tag runs `.github/workflows/release.yml`, which type-checks,
runs the tests, builds the installer and attaches it to a **draft** release.
Publishing it is a manual step on GitHub.

### Updater signing

The in-app updater needs each installer signed with a minisign key, which is
unrelated to Windows code signing and costs nothing. Two repository secrets
drive it:

- `TAURI_SIGNING_PRIVATE_KEY` — the **contents** of the private key file, not
  a path. Tauri ignores `TAURI_SIGNING_PRIVATE_KEY_PATH` here and fails the
  build with "a public key has been found, but no private key".
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — empty for the current key.

The matching public key is committed in `tauri.conf.json` under
`plugins.updater.pubkey`, and is baked into every build. **Losing the private
key permanently cuts off every installed copy from updates**, because a new
keypair will never match the public key already shipped. Regenerate with
`npm run tauri signer generate` only if you accept that.

Note this Tauri version signs the NSIS `setup.exe` directly and writes a
sibling `.sig`. It does not produce the `.nsis.zip` the v1 updater used, so
`latest.json` points at the `.exe`.

The installer is unsigned, so Windows SmartScreen warns on first run. A
code-signing certificate is the only fix and costs money.
