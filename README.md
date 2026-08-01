# SAM - Colony Edition

A Steam Achievement Manager for Linux, Windows, and macOS. Browse, unlock, and lock Steam achievements for any game in your library. Edit numeric statistics.

Fork of [jsnli/Samira](https://github.com/jsnli/Samira) — rebuilt with the Project Colony design language, ported to Svelte 5, fixed several upstream Rust bugs that prevented achievements from loading on games with > 32 achievements, and made cross-platform.

---

## Install

### Via Colony (recommended)

The easiest path. [Install Colony](https://github.com/Project-Colony/Colony#installation), then find **SAM - Colony Edition** in the app grid and click Install. Updates happen in-app.

### Direct download

Grab the single-file executable for your platform from the [latest release](https://github.com/Project-Colony/SAM-Colony-Edition/releases/latest):

| Platform               | Asset                                |
|------------------------|--------------------------------------|
| Linux (x86_64)         | `sam-colony-edition-linux`           |
| Windows (x86_64)       | `sam-colony-edition-windows.exe`     |
| macOS (Apple Silicon)  | `sam-colony-edition-macos`           |
| macOS (Intel)          | `sam-colony-edition-macos-x86`       |

No installer. Single self-contained executable per platform.

**Linux** — `chmod +x sam-colony-edition-linux && ./sam-colony-edition-linux`. The Steam runtime `.so` is embedded and unpacked to `~/.local/share/SAM-Colony-Edition/` on first launch.

**Windows** — Double-click the `.exe`. The Steam runtime DLL is embedded and unpacked to `%LOCALAPPDATA%\SAM-Colony-Edition\` on first launch (visible there afterward).

**macOS** — `chmod +x sam-colony-edition-macos`. On first launch macOS Gatekeeper will block because the binary is not signed by an Apple Developer account — bypass with `xattr -d com.apple.quarantine sam-colony-edition-macos` or right-click → Open → "Open Anyway" in Finder.

### Prerequisites

Steam must be running, you must be signed in, and you must own the game whose achievements you want to manage.

Steam installed via Flatpak is **not supported** — SAM reads from the Steam install dir on disk and Flatpak's sandboxing breaks that. Use your distro's native Steam package or the official installer from [steampowered.com](https://store.steampowered.com/about/).

---

## What changed vs upstream Samira

### Backend (Rust)
- **CallbackHandle leak** — `client.register_callback(...)` was discarded immediately, unregistering the callback before Steam could fire `UserStatsReceived`. Fix: bind the handle to a named local for the duration of the wait loop. Without this fix, achievements load empty for most games.
- **`break;` after first ACHIEVEMENTS block** — Steam splits >32 achievements across multiple bitfield blocks; the loop only read the first. Fix: iterate all blocks. Without this, big games (Elden Ring, Skyrim, etc.) lose half their achievement icons.
- **`.unwrap()` panic-on-failure** — any transient Steam API hiccup crashed the load with `catch_unwind` returning `Vec::new()` and the UI showing "0/0". Fix: `.unwrap_or(false)` / `.unwrap_or_default()` for graceful degradation.
- **Crash on zero-achievement games** — `get_achievement_names()` panics inside steamworks-rs when the game has no achievements. Fix: early-return on `get_num_achievements() == 0`.
- **Cross-platform paths** — `steam_root()` resolves Windows registry / macOS Application Support / Linux `~/.steam/steam` instead of hardcoded Linux path.

### Frontend
- **React → Svelte 5** — smaller bundle, less re-render churn, runes for cleaner reactivity.
- **H&E parchment theme** — replaces the dark Steam-style chrome with the Project Colony design language (parchment + burgundy + JetBrainsMono Nerd Font).
- **CSS containment + lazy images** — smooth scrolling on long achievement lists.

### Distribution
- Single-file portable per platform (no installer wizards, no archives).
- Steam runtime library embedded in the binary via `include_bytes!` and extracted on first launch (see `src-tauri/src/bootstrap.rs`). Lets us match Colony's single-asset download convention.
- GitHub Actions matrix builds + automatic VirusTotal scan + SHA256SUMS per platform.

---

## Build from source

```bash
git clone https://github.com/Project-Colony/SAM-Colony-Edition.git
cd SAM-Colony-Edition
npm install
npm run tauri build
```

Requires Rust 1.80+, Node 20+, and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

On Arch Linux a strip workaround is needed:
```bash
NO_STRIP=true npm run tauri build
```

---

## Credits

- [jsnli/Samira](https://github.com/jsnli/Samira) — the upstream this program is derived from, and the origin of most of what it still is. The VDF parser, the Steamworks integration, and the overall shape of the Rust backend are upstream's work, much of it unchanged here. Samira is GPL-3.0; this fork inherits those terms.
- [Valve](https://partner.steamgames.com/) — Steamworks SDK (redistributed per [SDK License](https://partner.steamgames.com/documentation/sdk_access_agreement)).
- [Project Colony](https://github.com/Project-Colony) — design language and distribution ecosystem.

## License

GPL-3.0. See `LICENSE`.

This is a derivative work of [jsnli/Samira](https://github.com/jsnli/Samira), which is
published under the GNU General Public License version 3. Substantial upstream code
survives here verbatim, so that licence carries forward: this fork is GPL-3.0 and cannot
be redistributed under anything more permissive. Earlier revisions of this README claimed
"MIT, matching upstream", which was true of neither project.

The SPDX identifier is the plain `GPL-3.0` rather than `GPL-3.0-or-later`. Upstream ships
the bare GPLv3 text with no "or any later version" notice, and section 14 of that licence
grants the option of following a later version only when the program itself says so.
Declaring `-or-later` would assert a permission upstream never gave, so the
version-neutral string is used instead.
