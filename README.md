<div align="center">

**A Steam achievement manager for Linux, Windows and macOS: unlock, lock and edit the achievements and statistics of the games you own.**

</div>

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Colony app](https://img.shields.io/badge/Colony-game-purple)](https://github.com/Project-Colony/Colony)
[![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20windows%20%7C%20macOS-lightgrey)](#installation)

Steam has no way to take back an achievement a bug unlocked, or to grant one a
broken game never awards. The classic Steam Achievement Manager runs on Windows
only, and Samira, the Linux manager this is forked from, loads no achievements
at all for most games. SAM - Colony Edition does the job on all three desktops,
from one self-contained executable that talks to the Steam client you are
already signed in to.

> **Status:** releases are built in CI and every asset is signed with the
> Project Colony release key since v0.7.0. CI builds the frontend and checks the
> Rust code on Linux, Windows and macOS, runs the unit tests on Linux and macOS,
> and starts the Linux and macOS executables on a machine without Valve's
> `libsteam_api` installed, since the v0.7.0 ones only started where it was.
> There are only a few unit tests and no UI tests, so achievement and statistic
> writes against a live Steam client are tested by hand. The macOS executables
> are not signed by an Apple Developer account, and Steam installed through
> Flatpak is not supported.

## Why SAM

- **[Steam Achievement Manager](https://github.com/gibbed/SteamAchievementManager)**
  is the original, and it is Windows only.
- **[Samira](https://github.com/jsnli/Samira)**, which this is a fork of, brings
  the idea to Linux, but loads no achievements for most games and no icons past the
  first 32 achievements (see [What changed from upstream](#what-changed-from-upstream)).
- **SAM - Colony Edition** runs on Linux, Windows and macOS, ships as a single
  executable per platform with no installer, and installs and updates through
  [Colony](https://github.com/Project-Colony/Colony), which checks every
  download's signature first.

## What it does

- **Find a game** by name or App ID. The search is fuzzy, over the list of every
  Steam app, and an App ID that is not in the list can still be opened.
- **Achievements:** every achievement of the game with its icon and its state.
  Filter by name or description, show only the locked or the unlocked ones,
  select or deselect them all, then apply the changes in one upload.
- **Statistics:** the game's integer statistics, each with its allowed range,
  edited in place and applied in one upload.
- **Steam's verdict, not a guess:** after an upload SAM waits for Steam to
  confirm it, reports what Steam rejected, and reloads the real values.
- Shows the game's header art and the Steam account in use.
- **Preferences:** click **SAM** in the top-left corner, or press `Ctrl+,`, for
  the language (English or French), the shared Project Colony themes and accent
  colours, the text size, and the accessibility options: high contrast, a
  dyslexia-friendly font, larger text and reduced motion. Changes apply and save
  at once. `Ctrl+1` and `Ctrl+2` switch between Achievements and Statistics, and
  `Esc` closes Preferences.

## What changed from upstream

- **Achievements load on every game.** Upstream dropped the Steam callback
  handle at once, so the stats never arrived, and read the icons of the first
  block of 32 achievements only.
- **No crash on games without achievements**, and transient Steamworks errors
  no longer empty the list.
- **Linux, Windows and macOS**, with the Steam install found per platform.
- **Svelte 5 instead of React**, on the Project Colony design system: the
  shared Colony themes, fonts and type scale, English and French, and the
  Colony Preferences page, with smooth scrolling on long lists.
- **One executable per platform:** the Steam runtime library is embedded in the
  binary and extracted on first launch (`src-tauri/src/bootstrap.rs`).

## Installation

SAM needs Steam running and signed in, and you must own the game whose
achievements you change. It reads achievement and statistic schemas from the
Steam install: the registry on Windows, `~/Library/Application Support/Steam` on
macOS, `~/.steam/steam` or `~/.local/share/Steam` on Linux. If Steam lives
anywhere else, set `STEAM_ROOT` to the directory that holds `appcache/`. Steam
installed through Flatpak is not supported: its sandbox keeps SAM out of the
Steam install.

### Via Colony (recommended)

Search for **SAM - Colony Edition** in
[Colony](https://github.com/Project-Colony/Colony) and install it. Updates
arrive through the launcher.

### Direct binary download

Grab the asset for your platform from the
[latest release](../../releases/latest):

| Platform | Asset |
|---|---|
| Linux | `sam-colony-edition-linux` |
| Windows | `sam-colony-edition-windows.exe` |
| macOS (Apple Silicon) | `sam-colony-edition-macos` |
| macOS (Intel) | `sam-colony-edition-macos-x86` |

```bash
chmod +x sam-colony-edition-linux && ./sam-colony-edition-linux
```

On first launch each version extracts Valve's Steam runtime library into
`steamworks/<version>/` in SAM's cache directory (see [Where SAM keeps its
files](#where-sam-keeps-its-files)).

The Windows executable has no Authenticode signature, so SmartScreen may warn
the first time. On macOS, Gatekeeper blocks the binary because Apple has not
signed it: run `xattr -d com.apple.quarantine sam-colony-edition-macos`, or
right-click it in Finder and choose Open.

### Build from source

```bash
git clone https://github.com/Project-Colony/SAM-Colony-Edition.git
cd SAM-Colony-Edition
npm ci
npm run tauri build -- --no-bundle
```

The executable is `src-tauri/target/release/samira` (`samira.exe` on Windows).
`npm run tauri dev` runs SAM from source with live reload.

The checks CI runs besides the build and the Rust tests:

```bash
npm run check:i18n      # English and French strings in step, every key defined
npm run check:style     # palette colours, scaled font sizes and shipped weights only
npm test                # palette logic: theme fallback, high contrast, text on the accent
npm run check:vendored  # themes, labels and fonts identical to Project-Colony-Resources
```

The Colony themes, the shared Preferences labels and the fonts are vendored
unchanged from
[Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources)
at the commit recorded in `scripts/vendored.json`. To move to a newer commit,
run `node scripts/check-vendored.mjs --update <commit>`.

Requires Rust 1.88 or newer, the current Node LTS, and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS. On
Debian and Ubuntu:

```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libsoup-3.0-dev
```

## Where SAM keeps its files

SAM follows the shared Project Colony layout, under a `Colony/SAM/` directory
in each platform root:

| What | Linux | Windows | macOS |
|---|---|---|---|
| Preferences: `preferences.json` | `~/.config/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\` | `~/Library/Application Support/Colony/SAM/` |
| WebView data: `webview/` | `~/.local/share/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\` | chosen by macOS, see below |
| Cache: `cache.json`, `steamworks/` | `~/.cache/Colony/SAM/` | `%LOCALAPPDATA%\Colony\SAM\cache\` | `~/Library/Caches/Colony/SAM/` |

On Linux the roots follow `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and
`XDG_CACHE_HOME` when they are set. The cache can be deleted at any time: SAM
downloads the app list and extracts the Steam runtime again. On macOS the
system's WebKit ignores the data directory SAM asks for and keeps the WebView
data under `~/Library/WebKit/` and `~/Library/Caches/`, in a folder macOS names
after the program.

Releases before this layout used `SAM-Colony-Edition/` directories (and
`ai.projectcolony.sam/` for the WebView). On first launch SAM moves `cache.json`
from the old cache directory, notes the move in an empty
`legacy-cache-migrated` file in its data directory so it happens once, and
extracts the Steam runtime again. If the move fails, SAM keeps using the old
`cache.json`. The old directories are left in place and can be deleted.

## Documentation

This README is the documentation for now; there is no `docs/` directory yet.
The source explains the parts that are not obvious, starting with the Steam
runtime extraction in `src-tauri/src/bootstrap.rs`. How Project Colony programs
are released and signed:
[design/releases.md](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/design/releases.md).

## Code signing policy

Every release asset is signed by the Project Colony organisation in CI, never on
a developer machine. Next to each asset on the release page:

| File | What it is |
|---|---|
| `<asset>.sig` | an ed25519 signature over the asset, made with the organisation's release key |
| `<asset>.meta` | three lines binding the asset to its file name, its sha256 and the release version |
| `<asset>.meta.sig` | an ed25519 signature over the `.meta` |

The private key is an organisation secret, used only by the shared
[sign-and-publish workflow](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/.github/workflows/sign-and-publish.yml)
in a job that builds nothing; the jobs that compile SAM never see it. Colony
checks all three files before it installs or updates SAM, refuses a release
older than the one installed, and refuses any asset whose signature does not
verify. To check a download yourself with OpenSSL 3 (the same commands work for
every asset):

```bash
cat > colony-release.pub <<'EOF'
-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEARNjg3Nn8H6/aBg1unwGjkUTcrdTxERNefVaqU8cFu0s=
-----END PUBLIC KEY-----
EOF
a=sam-colony-edition-linux
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a" -sigfile "$a.sig"
openssl pkeyutl -verify -pubin -inkey colony-release.pub -rawin -in "$a.meta" -sigfile "$a.meta.sig"
cat "$a.meta"     # version=<tag>, asset=<file name>, sha256=<digest>
sha256sum "$a"    # the digest must equal the sha256 line
```

SignPath is not used: SAM is not eligible for SignPath Foundation signing, so
the Windows executable carries these signatures and no Authenticode signature.
The macOS executables are not signed or notarised by Apple.

How releases are built, signed and published:
[design/releases.md](https://github.com/Project-Colony/Project-Colony-Resources/blob/main/design/releases.md#5-signing).

## Privacy

SAM sends no telemetry, no analytics and no crash reports. It has no account of
its own and never sees your Steam password: it acts through the Steam client
you are already signed in to.

| Data | Stored or sent | Where, and why |
|---|---|---|
| Steam app list | downloaded, then stored | At most once a day, SAM downloads `games_appid.json`, the ID and name of every Steam app, from `raw.githubusercontent.com`. It comes from [jsnli/steamappidlist](https://github.com/jsnli/steamappidlist), a personal repository of the upstream author that SAM follows unpinned. The request carries SAM's own User-Agent (`sam-colony-edition/<version>`) and nothing about you; GitHub sees your IP address ([GitHub's privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)). The list is cached in `cache.json`, below. |
| Game art and achievement icons | loaded from Valve's CDN | When you open a game, the window loads its header image from `shared.cloudflare.steamstatic.com` and its achievement icons from `cdn.cloudflare.steamstatic.com`, so Valve's CDN learns which game you opened ([Valve's privacy policy](https://store.steampowered.com/privacy_agreement/)). |
| SteamID and persona name | read from the local Steam client | Asked from the running Steam client through Valve's Steamworks library, to show which account is in use. SAM does not store or send them. |
| Achievements and statistics | written to your Steam account | The changes you apply go through the Steam client, which syncs them to Valve's servers. They appear on your profile like any others. |
| Play status | reported by Steam | Opening a game starts Steamworks as that game, so Steam shows you as playing it and records playtime while SAM has it open. |
| Steam files | read | The achievement and statistic schemas in Steam's `appcache/stats/`. On Windows, the Steam install path in `HKCU\Software\Valve\Steam`. |
| Preferences | stored | `preferences.json` in SAM's config directory, see [Where SAM keeps its files](#where-sam-keeps-its-files). It holds your Preferences choices (language, theme, accent, text sizes and the accessibility options) and nothing else. |
| App list cache | stored | `cache.json` in SAM's cache directory. |
| Steam runtime library | stored | Extracted on first launch into `steamworks/<version>/` in SAM's cache directory. |
| WebView data | stored | The window's web engine keeps its cache, including the images above, in `webview/` in SAM's data directory, and on macOS where the system's WebKit chooses. |

SAM itself connects to no server except the ones in this table, and only for
the action named there. The Steam client keeps talking to Valve as it always
does.

## Credits

- [jsnli/Samira](https://github.com/jsnli/Samira): the upstream this program is
  derived from, and the origin of most of what it still is. The VDF parser, the
  Steamworks integration and the shape of the Rust backend are upstream's work,
  much of it unchanged here.
- [Valve](https://partner.steamgames.com/): the Steamworks SDK, redistributed
  under the [Steamworks SDK Access Agreement](https://partner.steamgames.com/documentation/sdk_access_agreement).
- [Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources):
  the Colony themes, the shared Preferences labels and the design rules.
- The fonts, each under the SIL Open Font License 1.1 with its licence text in
  `assets/fonts/`: [JetBrains Mono](https://www.jetbrains.com/lp/mono/) by
  JetBrains, as patched by [Nerd Fonts](https://www.nerdfonts.com/), for the
  interface; [OpenDyslexic](https://opendyslexic.org/) for the dyslexia-friendly
  option; [Font Awesome Free](https://fontawesome.com/) for icons.

## License

GPL-3.0, SPDX identifier `GPL-3.0-only`. See [LICENSE](LICENSE).

This is a derivative work of [jsnli/Samira](https://github.com/jsnli/Samira),
which is published under the GNU General Public License version 3. Substantial
upstream code survives here verbatim, so that licence carries forward: this fork
cannot be redistributed under anything more permissive. Earlier revisions of
this README claimed "MIT, matching upstream", which was true of neither project.

The licence is GPL-3.0-only rather than GPL-3.0-or-later because upstream ships
the bare GPLv3 text with no "or any later version" notice, and section 14 of
that licence grants the option of following a later version only when the
program itself says so. Declaring "or later" would assert a permission upstream
never gave.

The executables also embed Valve's proprietary Steamworks runtime
(`steam_api64.dll`, `libsteam_api.so` or `libsteam_api.dylib`), which is
redistributed under the Steamworks SDK Access Agreement and is not covered by
the GPL.
