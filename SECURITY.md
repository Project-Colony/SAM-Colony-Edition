# Security policy

SAM - Colony Edition writes to your Steam account through the Steam client,
parses files from the Steam install, downloads data from the network and ships
signed executables. Security reports are welcome and handled with priority.

## Supported versions

Only the latest release receives security fixes. Colony offers each new release,
so staying current is one update away.

| Version | Supported |
|---|---|
| latest release | yes |
| anything older | no |

## Reporting a vulnerability

Report vulnerabilities privately through
[GitHub Security Advisories](https://github.com/Project-Colony/SAM-Colony-Edition/security/advisories/new)
("Report a vulnerability"). Do not open a public issue for an exploitable bug.

What to expect:

- an acknowledgement within a few days;
- a fix, or a plan to mitigate, before any public disclosure, coordinated with
  you;
- credit in the release notes if you want it.

## Scope

Reports of particular interest:

- **The VDF parser** (`src-tauri/src/vdf.rs`): it reads the binary achievement
  and statistic schemas in Steam's `appcache/stats/`. A crafted schema that
  crashes SAM, hangs it or exhausts its memory is in scope.
- **The app list download** (`src-tauri/src/dataset.rs`): SAM fetches
  `games_appid.json` from `raw.githubusercontent.com` and caches it in
  `cache.json`. Anything a hostile or tampered list, or cache file, can make SAM
  do beyond showing wrong names is in scope.
- **The release signing chain**: every asset ships with an ed25519 signature
  and a signed `.meta` sidecar, made by the shared workflow in
  [Project-Colony-Resources](https://github.com/Project-Colony/Project-Colony-Resources).
  A way to publish or install an unsigned, mismatched or older SAM through
  Colony is critical.
- **The Tauri IPC surface** (`src-tauri/src/commands.rs` and
  `src-tauri/capabilities/default.json`): the window may call SAM's own
  commands only, under a content security policy that admits Steam's CDN for
  images and nothing else. A way for web content to reach the file system, the
  network or the Steam client beyond those commands is in scope.

Out of scope: vulnerabilities in Steam, the Steamworks runtime or the games
themselves (report those to Valve or the game's developer), and Steam's own
rules about achievement and statistic changes.
