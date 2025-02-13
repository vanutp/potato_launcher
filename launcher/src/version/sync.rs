use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{debug, info, warn};
use rand::seq::SliceRandom as _;
use shared::adaptive_download::download_files;
use shared::paths::{
    get_authlib_injector_path, get_instance_dir, get_libraries_dir, get_natives_dir,
};
use shared::version::asset_metadata::AssetsMetadata;
use std::fs;
use tokio::fs as tokio_fs;
use zip::ZipArchive;

use shared::files::{self, CheckEntry};
use shared::progress::ProgressBar;
use shared::version::extra_version_metadata::{AuthBackend, ExtraVersionMetadata};
use shared::version::version_metadata;

use crate::lang::LangMessage;

use super::complete_version_metadata::CompleteVersionMetadata;
use super::os;

async fn get_objects_entries(
    extra_version_metadata: &ExtraVersionMetadata,
    force_overwrite: bool,
    instance_dir: &Path,
) -> anyhow::Result<Vec<CheckEntry>> {
    let include = &extra_version_metadata.include;

    let mut check_entries = vec![];
    let mut used_paths = HashSet::new();
    for rule in include {
        let objects = &rule.objects;

        let objects_paths = rule
            .objects
            .iter()
            .map(|object| instance_dir.join(&object.path))
            .collect::<HashSet<_>>();

        if rule.overwrite && rule.delete_extra || force_overwrite {
            let rule_path = instance_dir.join(&rule.path);
            let files_in_dir = files::get_files_ignore_paths(&rule_path, &used_paths)?;
            for file in files_in_dir {
                if !objects_paths.contains(&file) {
                    tokio_fs::remove_file(file).await?;
                }
            }
        }

        if rule.overwrite {
            check_entries.extend(objects.iter().map(|object| CheckEntry {
                url: object.url.clone(),
                remote_sha1: Some(object.sha1.clone()),
                path: instance_dir.join(&object.path),
            }));
        } else if rule.recursive || !instance_dir.join(&rule.path).exists() {
            check_entries.extend(objects.iter().filter_map(|object| {
                let path = instance_dir.join(&object.path);
                if !path.exists() {
                    Some(CheckEntry {
                        url: object.url.clone(),
                        remote_sha1: Some(object.sha1.clone()),
                        path,
                    })
                } else {
                    None
                }
            }));
        }

        used_paths.extend(objects_paths);
    }

    Ok(check_entries)
}

async fn fetch_hashes(
    sha1_urls: HashMap<PathBuf, String>,
) -> anyhow::Result<HashMap<PathBuf, String>> {
    let client = reqwest::Client::new();

    let mut futures = vec![];
    for (path, url) in sha1_urls {
        let client = client.clone();
        let future = async move {
            let response = client.get(&url).send().await?.error_for_status()?;
            let bytes = response.bytes().await?;
            let sha1 = String::from_utf8(bytes.to_vec())?;
            Ok((path, sha1))
        };
        futures.push(future);
    }

    let results: Vec<Result<(PathBuf, String), anyhow::Error>> =
        futures::future::join_all(futures).await;
    let mut hashes = HashMap::new();
    for result in results {
        match result {
            Ok((path, sha1)) => {
                hashes.insert(path, sha1);
            }
            Err(e) => {
                if let Some(se) = e.downcast_ref::<reqwest::Error>() {
                    if se.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                        warn!("SHA1 hash not found: {:?}", se.url().unwrap());
                    } else {
                        return Err(e);
                    }
                } else {
                    return Err(e);
                }
            }
        }
    }

    Ok(hashes)
}

async fn get_libraries_entries(
    libraries: &Vec<version_metadata::Library>,
    libraries_dir: &Path,
) -> anyhow::Result<Vec<CheckEntry>> {
    let mut sha1_urls = HashMap::<PathBuf, String>::new();
    let mut check_download_entries: Vec<CheckEntry> = Vec::new();

    for library in libraries {
        for entry in library.get_check_entries(
            libraries_dir,
            Some((&os::get_os_name(), &os::get_system_arch())),
        ) {
            if entry.remote_sha1.is_some() || !entry.path.exists() {
                if entry.url.is_empty() {
                    info!("Skipping library with no URL: {:?}", entry.path);
                    continue;
                }
                check_download_entries.push(entry);
            } else {
                sha1_urls.insert(entry.path.clone(), library.get_sha1_url());
                check_download_entries.push(CheckEntry {
                    remote_sha1: None,
                    ..entry
                });
            }
        }
    }

    let missing_hashes = fetch_hashes(sha1_urls).await?;

    let check_download_entries: Vec<_> = check_download_entries
        .into_iter()
        .map(|entry| {
            if entry.remote_sha1.is_none() {
                if let Some(sha1) = missing_hashes.get(&entry.path) {
                    return CheckEntry {
                        remote_sha1: Some(sha1.clone()),
                        ..entry
                    };
                }
            }
            entry
        })
        .collect();

    Ok(check_download_entries)
}

fn extract_natives(
    libraries: &Vec<version_metadata::Library>,
    libraries_dir: &Path,
    natives_dir: &Path,
) -> anyhow::Result<()> {
    for library in libraries {
        if let Some(natives_path) =
            library.get_os_native_path(libraries_dir, &os::get_os_name(), &os::get_system_arch())
        {
            extract_files(&natives_path, natives_dir)?;
        }
    }

    Ok(())
}

fn extract_files(src: &Path, dest: &Path) -> anyhow::Result<()> {
    let file = fs::File::open(src)?;
    let mut zip = ZipArchive::new(file)?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if let Some(file_path) = entry.enclosed_name() {
            let output_path = dest.join(file_path);
            if entry.is_file() {
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&output_path)?;
                std::io::copy(&mut entry, &mut outfile)?;
            } else if entry.is_dir() {
                fs::create_dir_all(&output_path)?;
            }
        }
    }

    Ok(())
}

pub const AUTHLIB_INJECTOR_URL: &str = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.5/authlib-injector-1.2.5.jar";
pub const AUTHLIB_INJECTOR_SHA1: &str = "1eca6aa7faf7ac6e3211862afa6e43fe2eedd07b";

fn get_authlib_injector_entry(
    version_metadata: &CompleteVersionMetadata,
    launcher_dir: &Path,
) -> Option<CheckEntry> {
    if let Some(auth_backend) = version_metadata.get_auth_backend() {
        if auth_backend == &AuthBackend::Microsoft {
            return None;
        }
    }

    Some(CheckEntry {
        url: AUTHLIB_INJECTOR_URL.to_string(),
        remote_sha1: Some(AUTHLIB_INJECTOR_SHA1.to_string()),
        path: get_authlib_injector_path(launcher_dir),
    })
}

pub async fn sync_instance(
    version_metadata: &CompleteVersionMetadata,
    force_overwrite: bool,
    launcher_dir: &Path,
    assets_dir: &Path,
    progress_bar: Arc<dyn ProgressBar<LangMessage> + Send + Sync>,
) -> anyhow::Result<()> {
    let version_name = version_metadata.get_name();

    let libraries_dir = get_libraries_dir(launcher_dir);
    let natives_dir = get_natives_dir(launcher_dir, version_metadata.get_parent_id());
    let instance_dir = get_instance_dir(launcher_dir, version_name);

    let mut check_entries = vec![];

    check_entries.push(version_metadata.get_client_check_entry(launcher_dir)?);

    let mut libraries = version_metadata.get_libraries_with_overrides();
    libraries.extend(version_metadata.get_extra_forge_libs());
    check_entries.extend(get_libraries_entries(&libraries, &libraries_dir).await?);

    if let Some(extra) = version_metadata.get_extra() {
        check_entries.extend(get_objects_entries(extra, force_overwrite, &instance_dir).await?);
    }

    if let Some(authlib_injector) = get_authlib_injector_entry(version_metadata, launcher_dir) {
        check_entries.push(authlib_injector);
    }

    let asset_index = version_metadata.get_asset_index()?;
    let asset_metadata = AssetsMetadata::read_or_download(asset_index, assets_dir).await?;

    check_entries.extend(asset_metadata.get_check_entries(
        assets_dir,
        version_metadata.get_resources_url_base(),
        force_overwrite,
    )?);

    info!("Got {} check download entries", check_entries.len());
    progress_bar.set_message(LangMessage::CheckingFiles);
    let mut download_entries =
        files::get_download_entries(check_entries, progress_bar.clone()).await?;

    let rng = &mut rand::rngs::OsRng;
    download_entries.shuffle(rng);

    info!("Got {} download entries", download_entries.len());

    let paths = download_entries
        .iter()
        .map(|x| x.path.clone())
        .collect::<Vec<_>>();
    debug!("Paths to download: {:?}", paths);

    progress_bar.set_message(LangMessage::DownloadingFiles);
    download_files(download_entries, progress_bar).await?;

    extract_natives(&libraries, &libraries_dir, &natives_dir)?;

    Ok(())
}
