# locale-remulator-wasm-plugin

A `WrapperPlugin` for Game Library Client implemented as a WASM component, ported from that
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

## Building

```sh
rustup target add wasm32-wasip1   # once
cargo install cargo-component     # once
cargo component build
```

Output: `target/wasm32-wasip1/debug/locale_remulator_wasm_plugin.wasm`.

## Installing into a running Game Library Client

Copy the compiled `.wasm` and `plugin.json` into
`<app data dir>/wasm-plugins/locale-remulator-wasm/` (Windows:
`%APPDATA%\com.minho.tauri-app\wasm-plugins\locale-remulator-wasm\`). It'll show up in
Settings' Plugins panel under the Wrapper tab next time the app starts, as
**Locale Remulator (WASM)** - clicking "Install" there (in the plugin's own settings row)
downloads and installs the real thing.
