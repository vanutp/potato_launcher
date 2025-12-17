use egui::ViewportBuilder;
use log::info;
use serde::Deserialize;

use crate::config::build_config;
use crate::constants::XMX_DEFAULT;
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
    let temp_dir = temp_dir.join(build_config::get_lower_launcher_name());
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

pub fn add_icon(builder: ViewportBuilder) -> ViewportBuilder {
    let Some(icon_bytes) = build_config::LAUNCHER_ICON else {
        return builder;
    };
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to open icon path")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    builder.with_icon(egui::IconData {
        width,
        height,
        rgba,
    })
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
        .join(build_config::get_lower_launcher_name());
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

lazy_static::lazy_static! {
    static ref total_memory: Option<u64> = sys_info::mem_info().ok().map(|mem_info| mem_info.total);
}

pub fn get_total_memory() -> Option<u64> {
    *total_memory
}

pub fn map_range(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    (value - from_min) / (from_max - from_min) * (to_max - to_min) + to_min
}

pub fn format_xmx(xmx: Option<&str>) -> String {
    let mut xmx_mb = XMX_DEFAULT;
    if let Some(xmx) = xmx {
        if xmx.ends_with('M') || xmx.ends_with('m') {
            let xmx = xmx.trim_end_matches(['M', 'm']);
            if let Ok(xmx) = xmx.parse::<u64>() {
                xmx_mb = xmx;
            }
        } else if xmx.ends_with('G') || xmx.ends_with('g') {
            let xmx = xmx.trim_end_matches(['G', 'g']);
            if let Ok(xmx) = xmx.parse::<u64>() {
                xmx_mb = xmx * 1024;
            }
        } else if let Ok(xmx) = xmx.parse::<u64>() {
            xmx_mb = xmx;
        }
    }

    if let Some(memory) = get_total_memory() {
        let memory_mb = memory / 1024;
        if xmx_mb > memory_mb {
            xmx_mb = memory_mb;
        }
    }

    format!("{xmx_mb}M")
}
