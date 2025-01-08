use std::path::{Path, PathBuf};

pub fn get_assets_dir(output_dir: &Path) -> PathBuf {
    let assets_dir = output_dir.join("assets");
    if !assets_dir.exists() {
        std::fs::create_dir_all(&assets_dir).unwrap();
    }
    assets_dir
}

pub fn get_replaced_metadata_dir(output_dir: &Path) -> PathBuf {
    let replaced_manifests_dir = output_dir.join("versions_replaced");
    if !replaced_manifests_dir.exists() {
        std::fs::create_dir_all(&replaced_manifests_dir).unwrap();
    }
    replaced_manifests_dir
}

pub async fn exec_string_command(command: &str) -> anyhow::Result<()> {
    let parts = shell_words::split(command)?;
    let mut cmd = tokio::process::Command::new(&parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    let status = cmd.status().await?;
    if !status.success() {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Command failed").into());
    }
    Ok(())
}
