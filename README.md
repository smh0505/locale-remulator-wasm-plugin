# locale-remulator-wasm-plugin

A `WrapperPlugin` for [Concourse](https://github.com/smh0505/Concourse) implemented as a WASM component, ported from that
project's built-in `locale_remulator.rs` (list/launch) and `wrapper_installer.rs` (download/
install) - both since deleted from the host app now that this plugin owns that logic entirely.
Fully self-contained: downloads the latest release from GitHub, extracts it, seeds a default
`LRConfig.xml` if none exists, and runs the real `LRInstaller.exe` (visible, for its
right-click context-menu registration), all into its own `plugin-dir()` - no host-side install
directory, download command, or path setting involved anymore.

This is a real, separate repo on purpose - same reasoning as `steam-source-wasm-plugin`/
`gog-source-wasm-plugin`: a plugin whose source lives inside the host app's own repo isn't
genuinely exercising the "install arbitrary third-party code" model the WASM plugin system is
for.

## Permissions

Declares the `run-programs` capability (`plugin.json`'s `capabilities` field) - both
`install()` (runs `LRInstaller.exe` via `host::run-and-wait`) and `launch()` (runs the
configured game via `host::spawn-process`) need it. Concourse gates both host functions behind
an explicit, visible per-plugin grant (Milestone 13). Wrapper plugins aren't install-by-URL
yet (see above), so this always shows as a "Permission needed" row with a Grant button in
Settings' Wrapper tab until granted once.

Also declares `httpScopes: ["github.com"]` (Milestone 13 URL allowlisting) - covers both
`api.github.com` (release lookup) and the actual download, whose target URL is dynamic
(`asset.browser_download_url`, not a literal in source) but always GitHub-issued, never
attacker-influenced input.

## Building

```sh
rustup target add wasm32-wasip1   # once
cargo install cargo-component     # once
cargo component build
```

Output: `target/wasm32-wasip1/debug/locale_remulator_wasm_plugin.wasm`.

## Installing into a running Concourse

Either build locally (above) or grab the prebuilt `.wasm` + `plugin.json` from this repo's
[Releases](https://github.com/smh0505/locale-remulator-wasm-plugin/releases) - CI (`.github/workflows/publish.yml`) publishes a new release
automatically whenever `plugin.json`'s `version` is bumped on `main`. Wrapper-kind plugins
don't support install-by-URL in Concourse yet (only source plugins, metadata plugins, and themes do) - the
latest manifest is still linkable directly, for reference or scripting a manual copy:

```
https://github.com/smh0505/locale-remulator-wasm-plugin/releases/latest/download/plugin.json
```

Copy the compiled `.wasm` and `plugin.json` into
`<app data dir>/wasm-plugins/wrapper/locale-remulator-wasm/` (Windows:
`%APPDATA%\com.bloppy.concourse\wasm-plugins\wrapper\locale-remulator-wasm\`). It'll show up in
Settings' Plugins panel under the Wrapper tab next time the app starts, as
**Locale Remulator** - clicking "Install" there (in the plugin's own settings row)
downloads and installs the real thing.

## Versioning

Plain SemVer (`Cargo.toml` + `plugin.json`'s `version`), independent of Concourse's own
milestone-tracked version - patch for fixes, minor for backward-compatible new capabilities,
major for breaking manifest/WIT interface changes. Full convention:
[`.claude/CLAUDE.md`](https://github.com/smh0505/Concourse/blob/main/.claude/CLAUDE.md) (Plugin Versioning) in the main [Concourse](https://github.com/smh0505/Concourse) repo.
