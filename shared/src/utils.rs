use std::path::Path;

use crate::version::version_manifest::{VersionInfo, VersionManifest};

pub const VANILLA_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(thiserror::Error, Debug)]
pub enum VanillaGeneratorError {
    #[error("Vanilla version not found")]
    VersionNotFound,
}

pub fn get_vanilla_version_info(
    version_manifest: &VersionManifest,
    minecraft_version: &str,
) -> anyhow::Result<VersionInfo> {
    let version_info = version_manifest
        .versions
        .iter()
        .find(|v| v.id == minecraft_version)
        .ok_or(VanillaGeneratorError::VersionNotFound)?;
    Ok(version_info.clone())
}

pub fn url_from_rel_path(rel_path: &Path, download_server_base: &str) -> anyhow::Result<String> {
    let path_str = rel_path.to_string_lossy().replace('\\', "/");

    Ok(format!(
        "{}/{}",
        download_server_base.trim_end_matches('/'),
        path_str
    ))
}

pub fn url_from_path(
    path: &Path,
    base_dir: &Path,
    download_server_base: &str,
) -> anyhow::Result<String> {
    let rel_path = path.strip_prefix(base_dir)?;
    url_from_rel_path(rel_path, download_server_base)
}

pub fn is_connect_error(e: &anyhow::Error) -> bool {
    if let Some(e) = e.downcast_ref::<reqwest::Error>() {
        return e.is_connect() || e.status().is_some_and(|s| s.as_u16() == 523);
        // 523 = Cloudflare Origin is Unreachable
    }

    // Check for connection-related error messages that cannot be checked by reqwest
    let error_str = format!("{:?}", e);
    error_str.contains("peer closed connection without sending TLS close_notify")
        || error_str.contains("connection closed")
        || error_str.contains("connection reset")
        || error_str.contains("connection aborted")
        || error_str.contains("broken pipe")
        || error_str.contains("SendRequest")
        || error_str.contains("connection error")
        || error_str.contains("Connection refused")
        || error_str.contains("Network is unreachable")
        || error_str.contains("Connection timed out")
}
