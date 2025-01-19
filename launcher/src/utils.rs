use log::info;
use serde::Deserialize;

use crate::config::build_config;
use crate::constants;
use std::fs;
use std::path::PathBuf;

pub fn set_sigint_handler() {
    ctrlc::set_handler(move || {
        info!("Exiting...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}

pub fn get_temp_dir() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let temp_dir = temp_dir.join(build_config::get_data_launcher_name());
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).unwrap();
    }
    temp_dir
}

pub fn is_read_only_error(e: &anyhow::Error) -> bool {
    if let Some(e) = e.downcast_ref::<std::io::Error>() {
        return e.kind() == std::io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(18);
    }
    false
}

pub fn is_connect_error(e: &anyhow::Error) -> bool {
    if let Some(e) = e.downcast_ref::<reqwest::Error>() {
        return e.is_connect() || e.status().is_some_and(|s| s.as_u16() == 523);
        // 523 = Cloudflare Origin is Unreachable
    }
    false
}

pub fn validate_xmx(xmx: &str) -> bool {
    let xmx = xmx.trim();
    if xmx.is_empty() {
        return false;
    }

    let xmx = xmx.to_uppercase();
    if xmx.ends_with("M") {
        if let Ok(mb) = xmx[..xmx.len() - 1].parse::<u32>() {
            return (constants::MIN_JAVA_MB..=constants::MAX_JAVA_MB).contains(&mb);
        }
    } else if xmx.ends_with("G") {
        if let Ok(gb) = xmx[..xmx.len() - 1].parse::<u32>() {
            return (constants::MIN_JAVA_MB..=constants::MAX_JAVA_MB).contains(&(gb * 1024));
        }
    }

    false
}

pub fn get_icon_data() -> egui::IconData {
    let image = image::load_from_memory(build_config::LAUNCHER_ICON)
        .expect("Failed to open icon path")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    egui::IconData {
        width,
        height,
        rgba,
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Single(T),
    Vec(Vec<T>),
}

impl<T> From<SingleOrVec<T>> for Vec<T> {
    fn from(single_or_vec: SingleOrVec<T>) -> Vec<T> {
        match single_or_vec {
            SingleOrVec::Single(single) => vec![single],
            SingleOrVec::Vec(vec) => vec,
        }
    }
}

pub fn get_data_dir() -> PathBuf {
    let data_dir = dirs::data_dir()
        .expect("Failed to get data directory")
        .join(build_config::get_data_launcher_name());
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    }
    data_dir
}

pub fn is_valid_minecraft_username(username: &str) -> bool {
    if username.len() < 3 || username.len() > 16 {
        return false;
    }
    for c in username.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }
    true
}
