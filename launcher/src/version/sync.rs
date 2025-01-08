use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use log::{debug, info, warn};
use shared::paths::{
    get_authlib_injector_path, get_instance_dir, get_libraries_dir, get_natives_dir,
};
use shared::version::asset_metadata::AssetsMetadata;
use std::fs;
use zip::ZipArchive;

use shared::files::{self, CheckEntry};
use shared::progress::ProgressBar;
use shared::version::extra_version_metadata::{AuthBackend, ExtraVersionMetadata};
use shared::version::version_metadata;

use crate::lang::LangMessage;

use super::complete_version_metadata::CompleteVersionMetadata;
use super::os;

fn get_objects_entries(
    extra_version_metadata: &ExtraVersionMetadata,
    force_overwrite: bool,
    instance_dir: &Path,
) -> anyhow::Result<Vec<CheckEntry>> {
    let objects = &extra_version_metadata.objects;
    let include = &extra_version_metadata.include;
    let include_no_overwrite = &extra_version_metadata.include_no_overwrite;

    let get_instance_files = |x| files::get_files_in_dir(&instance_dir.join(x)).ok();
    let no_overwrite_iter = include_no_overwrite
        .iter()
        .filter_map(get_instance_files)
        .flatten();
    let mut to_overwrite: HashSet<PathBuf> = include
        .iter()
        .filter_map(get_instance_files)
        .flatten()
        .collect();
    let mut no_overwrite = HashSet::new();
    if !force_overwrite {
        no_overwrite.extend(no_overwrite_iter);
    } else {
        to_overwrite.extend(no_overwrite_iter);
    }

    // Remove files that are in both no_overwrite and overwrite
    // e.g. config folder is in no_overwrite but config/<filename>.json is in overwrite
    no_overwrite.retain(|x| !to_overwrite.contains(x));

    // delete extra to_overwrite files
    let objects_hashset: HashSet<PathBuf> =
        objects.iter().map(|x| instance_dir.join(&x.path)).collect();
    let _ = to_overwrite
        .iter()
        .map(|x| {
            if !objects_hashset.contains(x) {
                fs::remove_file(x).unwrap();
            }
        })
        .collect::<Vec<()>>();

    let mut download_entries = vec![];
    for object in objects.iter() {
        let object_path = instance_dir.join(&object.path);

        if no_overwrite.contains(&object_path) {
            continue;
        }
        download_entries.push(CheckEntry {
            url: object.url.clone(),
            remote_sha1: Some(object.sha1.clone()),
            path: object_path,
        });
    }

    Ok(download_entries)
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
                        return Err(e.into());
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
                if entry.url == "" {
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
            let exclude = library.get_extract().map(|x| {
                x.exclude
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<HashSet<_>>()
            });
            extract_files(&natives_path, &natives_dir, exclude)?;
        }
    }

    Ok(())
}

fn extract_files(src: &Path, dest: &Path, exclude: Option<HashSet<String>>) -> anyhow::Result<()> {
    let exclude = exclude.unwrap_or_default();

    let file = fs::File::open(src)?;
    let mut zip = ZipArchive::new(file)?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        if let Some(file_path) = entry.enclosed_name() {
            if let Some(directory) = file_path.components().next() {
                let directory = directory.as_os_str().to_str().unwrap_or_default();
                if exclude.contains(directory)
                    || exclude.contains(format!("{}/", directory).as_str())
                {
                    continue;
                }
            }

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

    return Some(CheckEntry {
        url: AUTHLIB_INJECTOR_URL.to_string(),
        remote_sha1: Some(AUTHLIB_INJECTOR_SHA1.to_string()),
        path: get_authlib_injector_path(launcher_dir),
    });
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
    let instance_dir = get_instance_dir(launcher_dir, &version_name);

    let mut check_entries = vec![];

    check_entries.push(version_metadata.get_client_check_entry(launcher_dir)?);

    let mut libraries = version_metadata.get_libraries_with_overrides();
    libraries.extend(version_metadata.get_extra_forge_libs());
    check_entries.extend(get_libraries_entries(&libraries, &libraries_dir).await?);

    if let Some(extra) = version_metadata.get_extra() {
        check_entries.extend(get_objects_entries(extra, force_overwrite, &instance_dir)?);
    }

    if let Some(authlib_injector) = get_authlib_injector_entry(version_metadata, launcher_dir) {
        check_entries.push(authlib_injector);
    }

    let asset_index = version_metadata.get_asset_index()?;
    let asset_metadata = AssetsMetadata::read_or_download(asset_index, assets_dir).await?;

    check_entries.extend(
        asset_metadata.get_check_entries(assets_dir, version_metadata.get_resources_url_base())?,
    );

    info!("Got {} check download entries", check_entries.len());
    progress_bar.set_message(LangMessage::CheckingFiles);
    let download_entries = files::get_download_entries(check_entries, progress_bar.clone()).await?;

    info!("Got {} download entries", download_entries.len());

    let paths = download_entries
        .iter()
        .map(|x| x.path.clone())
        .collect::<Vec<_>>();
    debug!("Paths to download: {:?}", paths);

    progress_bar.set_message(LangMessage::DownloadingFiles);
    files::download_files(download_entries, progress_bar).await?;

    extract_natives(&libraries, &libraries_dir, &natives_dir)?;

    Ok(())
}
