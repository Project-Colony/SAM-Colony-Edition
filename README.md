# SAM - Colony Edition

A Steam Achievement Manager for Linux, Windows, and macOS. Browse, unlock, and lock Steam achievements for any game in your library. Edit numeric statistics.

Fork of [jsnli/Samira](https://github.com/jsnli/Samira), rebuilt on the Project Colony design system, ported to Svelte 5, fixed several upstream Rust bugs that prevented achievements from loading on games with > 32 achievements, and made cross-platform.

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

**Linux**: `chmod +x sam-colony-edition-linux && ./sam-colony-edition-linux`. The Steam runtime `.so` is embedded and unpacked to `~/.cache/Colony/SAM/steamworks/<version>/` on first launch.

**Windows**: Double-click the `.exe`. The Steam runtime DLL is embedded and unpacked to `%LOCALAPPDATA%\Colony\SAM\cache\steamworks\<version>\` on first launch (visible there afterward).

**macOS**: `chmod +x sam-colony-edition-macos`. The Steam runtime `.dylib` is embedded and unpacked to `~/Library/Caches/Colony/SAM/steamworks/<version>/` on first launch. On first launch macOS Gatekeeper will block because the binary is not signed by an Apple Developer account. Bypass with `xattr -d com.apple.quarantine sam-colony-edition-macos` or right-click → Open → "Open Anyway" in Finder.

### Prerequisites

Steam must be running, you must be signed in, and you must own the game whose achievements you want to manage.

SAM reads achievement icons and statistics from the Steam install directory. It looks in the default place for each OS (the registry on Windows, `~/Library/Application Support/Steam` on macOS, `~/.steam/steam` or `~/.local/share/Steam` on Linux). If your Steam lives somewhere else, set `STEAM_ROOT` to the directory that holds `appcache/`.

Steam installed via Flatpak is **not supported**: SAM reads from the Steam install dir on disk and Flatpak's sandboxing breaks that. Use your distro's native Steam package or the official installer from [steampowered.com](https://store.steampowered.com/about/).

### Preferences

Click **SAM** in the top-left corner, or press `Ctrl+,`, to open Preferences: the language (English or French), the shared Project Colony themes and accent colours, the text size, and the accessibility options (high contrast, a dyslexia-friendly font, larger text, reduced motion). Changes apply and save immediately. `Ctrl+1` and `Ctrl+2` switch between Achievements and Statistics, and `Esc` closes Preferences.

### Where SAM keeps its files

SAM follows the shared Project Colony layout, under a `Colony/SAM/` directory:

| What | Linux | Windows | macOS |
|------|-------|---------|-------|
| Preferences (`preferences.json`) | `~/.config/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\` | `~/Library/Application Support/Colony/SAM/` |
| WebView data (`webview/`) | `~/.local/share/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\` | managed by macOS, see below |
| Cache: games list (`cache.json`), Steam runtime (`steamworks/`) | `~/.cache/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\cache\` | `~/Library/Caches/Colony/SAM/` |

The cache can be deleted at any time: SAM downloads the games list and unpacks the Steam runtime again. On macOS the system WebView (WKWebView) ignores the data directory SAM asks for and keeps its data in the location macOS chooses for the app.

Releases before this layout used `SAM-Colony-Edition/` directories instead. On first launch SAM moves the games list from there and re-unpacks the Steam runtime; the old directory is left in place and can be deleted.

### Privacy

SAM has no account, no telemetry and no analytics. It stores only the files above, on your machine. `preferences.json` holds your Preferences choices and nothing else. The network requests it makes are to Steam through the Steam client, to `raw.githubusercontent.com` once a day for the list of Steam games, and to Steam's image servers for game headers and achievement icons.

---

## What changed vs upstream Samira

### Backend (Rust)
- **CallbackHandle leak**: `client.register_callback(...)` was discarded immediately, unregistering the callback before Steam could fire `UserStatsReceived`. Fix: bind the handle to a named local for the duration of the wait loop. Without this fix, achievements load empty for most games.
- **`break;` after first ACHIEVEMENTS block**: Steam splits >32 achievements across multiple bitfield blocks; the loop only read the first. Fix: iterate all blocks. Without this, big games (Elden Ring, Skyrim, etc.) lose half their achievement icons.
- **`.unwrap()` panic-on-failure**: any transient Steam API hiccup crashed the load with `catch_unwind` returning `Vec::new()` and the UI showing "0/0". Fix: `.unwrap_or(false)` / `.unwrap_or_default()` for graceful degradation.
- **Crash on zero-achievement games**: `get_achievement_names()` panics inside steamworks-rs when the game has no achievements. Fix: early-return on `get_num_achievements() == 0`.
- **Cross-platform paths**: `steam_root()` resolves Windows registry / macOS Application Support / Linux `~/.steam/steam` instead of hardcoded Linux path.

### Frontend
- **React → Svelte 5**: smaller bundle, less re-render churn, runes for cleaner reactivity.
- **Project Colony design system**: the shared Colony themes and accents, the Colony fonts and type scale, English and French, and the Colony Preferences page with its accessibility options.
- **CSS containment + lazy images**: smooth scrolling on long achievement lists.

### Distribution
- Single-file portable per platform (no installer wizards, no archives).
- Steam runtime library embedded in the binary via `include_bytes!` and extracted on first launch (see `src-tauri/src/bootstrap.rs`). Lets us match Colony's single-asset download convention.
- GitHub Actions matrix builds released through release-please; every asset is signed with the Project Colony release key (`.sig` plus a signed `.meta` sidecar) and scanned on VirusTotal.

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

Checks that CI also runs:
```bash
npm run check:i18n      # English and French strings in step, every key defined
npm run check:style     # palette colours, scaled font sizes and shipped weights only
npm test                # palette logic (theme fallback, high contrast, text on accent)
npm run check:vendored  # themes, labels and fonts match Project-Colony-Resources
```

The themes, the shared Preferences labels and the fonts are vendored unchanged from [Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources) at the commit recorded in `scripts/vendored.json`. To update them, run `node scripts/check-vendored.mjs --update <commit>`.

---

## Credits

- [jsnli/Samira](https://github.com/jsnli/Samira): the upstream this program is derived from, and the origin of most of what it still is. The VDF parser, the Steamworks integration, and the overall shape of the Rust backend are upstream's work, much of it unchanged here. Samira is GPL-3.0; this fork inherits those terms.
- [Valve](https://partner.steamgames.com/): Steamworks SDK (redistributed per [SDK License](https://partner.steamgames.com/documentation/sdk_access_agreement)).
- [Project Colony](https://github.com/Project-Colony): design system and distribution ecosystem.
- Fonts, each under the SIL Open Font License 1.1 with its licence text in `assets/fonts/`: [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono) patched by [Nerd Fonts](https://github.com/ryanoasis/nerd-fonts), [OpenDyslexic](https://github.com/antijingoist/opendyslexic) and [Font Awesome Free](https://github.com/FortAwesome/Font-Awesome).

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
