//! WASM `WrapperPlugin` for Locale Remulator - fully self-contained (Milestone 10's WASM
//! migration follow-up): manages its own download/install into `plugin-dir()`, not just the
//! launch/list-profiles logic. Ported from the game-library-client's built-in
//! `wrapper_installer.rs` (install) and `locale_remulator.rs` (list/launch), both since
//! deleted from the host app now that this plugin owns that logic entirely.

#[allow(warnings)]
mod bindings;

use bindings::exports::gamelib::plugin::wrapper_plugin::{Guest, LocaleProfile};
use bindings::gamelib::plugin::host;
use serde::Deserialize;

struct LocaleRemulatorPlugin;

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize)]
struct GitHubRelease {
    assets: Vec<GitHubAsset>,
}

/// `Locale_Remulator` ships exactly one `.zip` asset per release - no per-arch variant to
/// disambiguate (confirmed against the real releases when this was still host-side code).
fn latest_release_asset() -> Result<GitHubAsset, String> {
    let json =
        host::http_get("https://api.github.com/repos/InWILL/Locale_Remulator/releases/latest")?;
    let release: GitHubRelease =
        serde_json::from_str(&json).map_err(|e| format!("Failed to parse release info: {}", e))?;
    release
        .assets
        .into_iter()
        .find(|asset| asset.name.ends_with(".zip"))
        .ok_or_else(|| "No .zip asset found in the latest release".to_string())
}

fn install_dir() -> Result<String, String> {
    Ok(format!("{}/install", host::plugin_dir()?))
}

fn proc_path() -> Result<String, String> {
    Ok(format!("{}/LRProc.exe", install_dir()?))
}

fn default_config_xml() -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<LRConfig>
  <Profiles Type="exe">
    <Profile Name="Japanese" Guid="{}">
      <Location>ja-JP</Location>
      <CodePage>932</CodePage>
      <LCID>1041</LCID>
      <TimeZone>Tokyo Standard Time</TimeZone>
      <Bias>540</Bias>
      <RunAsAdmin>false</RunAsAdmin>
      <HookIME>false</HookIME>
      <HookLCID>true</HookLCID>
    </Profile>
    <Profile Name="Japanese (Admin)" Guid="{}">
      <Location>ja-JP</Location>
      <CodePage>932</CodePage>
      <LCID>1041</LCID>
      <TimeZone>Tokyo Standard Time</TimeZone>
      <Bias>540</Bias>
      <RunAsAdmin>true</RunAsAdmin>
      <HookIME>true</HookIME>
      <HookLCID>true</HookLCID>
    </Profile>
  </Profiles>
</LRConfig>
"#,
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    )
}

/// Neither the release zip ships `LRConfig.xml` (only created by `LREditor.exe`'s own GUI,
/// which the managed install never runs) nor does anything else generate one - so this seeds
/// a minimal default profile pair with freshly generated GUIDs. Never overwrites an existing
/// config (e.g. one a real install left behind).
fn seed_default_config_if_missing() -> Result<(), String> {
    let config_path = format!("{}/LRConfig.xml", install_dir()?);
    if host::path_exists(&config_path) {
        return Ok(());
    }
    host::write_file(&config_path, &default_config_xml())
}

fn parse_profiles(content: &str) -> Result<Vec<LocaleProfile>, String> {
    let doc = roxmltree::Document::parse(content).map_err(|e| e.to_string())?;
    Ok(doc
        .descendants()
        .filter(|n| n.has_tag_name("Profile"))
        .filter_map(|n| {
            Some(LocaleProfile {
                name: n.attribute("Name")?.to_string(),
                guid: n.attribute("Guid")?.to_string(),
            })
        })
        .collect())
}

impl Guest for LocaleRemulatorPlugin {
    fn install() -> Result<(), String> {
        let asset = latest_release_asset()?;
        let bytes = host::download_bytes(&asset.browser_download_url)?;
        if bytes.len() as u64 != asset.size {
            return Err(format!(
                "Download size mismatch: expected {} bytes, got {} - the download may be corrupted or incomplete",
                asset.size,
                bytes.len()
            ));
        }

        let plugin_dir = host::plugin_dir()?;
        let staging_dir = format!("{}/.staging", plugin_dir);
        host::extract_zip(&bytes, &staging_dir)?;

        // The real release zip wraps its contents in one top-level folder matching the
        // archive name (e.g. `Locale_Remulator.1.6.0/LRProc.exe`) - unwrap it so install_dir()
        // ends up holding LRProc.exe directly.
        let content_dir = host::unwrap_single_subdir(&staging_dir)?;
        let dest_dir = install_dir()?;
        host::replace_dir(&content_dir, &dest_dir)?;
        if content_dir != staging_dir {
            let _ = host::remove_dir(&staging_dir);
        }

        seed_default_config_if_missing()?;

        // LRInstaller.exe only registers an optional right-click context-menu shell
        // extension - LRProc.exe itself runs standalone without it - but it's run anyway for
        // one consistent install experience with Locale Emulator (which genuinely needs its
        // own installer's GAC registration step). Its window shows in front of the user, same
        // as if they'd downloaded and run it manually.
        host::run_and_wait(&format!("{}/LRInstaller.exe", dest_dir), &[], &dest_dir)
    }

    fn uninstall() -> Result<(), String> {
        host::remove_dir(&install_dir()?)
    }

    fn is_installed() -> bool {
        proc_path().map(|p| host::path_exists(&p)).unwrap_or(false)
    }

    fn list_profiles() -> Result<Vec<LocaleProfile>, String> {
        let config_path = format!("{}/LRConfig.xml", install_dir()?);
        let content = host::read_file(&config_path)?;
        parse_profiles(&content)
    }

    fn launch(profile_guid: String, executable_path: String) -> Result<(), String> {
        host::spawn_process(&proc_path()?, &[profile_guid, executable_path])
    }
}

bindings::export!(LocaleRemulatorPlugin with_types_in bindings);
