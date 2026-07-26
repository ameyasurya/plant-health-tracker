use std::path::Path;

fn main() {
    // tauri-build embeds the app icon into a Windows resource here in the
    // build script, but it only declares rerun-if-changed for
    // tauri.conf.json and the capabilities directory. Regenerating the
    // icons therefore leaves Cargo with no reason to re-run this script,
    // so the binary relinks against the previous resource and quietly
    // ships the old icon.
    //
    // This matters most in CI, where the target directory is restored from
    // cache: a tagged release could otherwise publish installers carrying
    // an icon that no longer exists in the repository.
    watch_icons(Path::new("icons"));
    println!("cargo:rerun-if-changed=build.rs");

    tauri_build::build()
}

fn watch_icons(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            watch_icons(&path);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
