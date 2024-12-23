use std::{path::Path, sync::Arc};

use log::error;
use shared::version::version_manifest::VersionInfo;
use tokio::runtime::Runtime;

use crate::{
    config::runtime_config::Config, version::complete_version_metadata::CompleteVersionMetadata,
};

use super::background_task::{BackgroundTask, BackgroundTaskResult};

#[derive(PartialEq)]
enum GetStatus {
    Getting,
    UpToDate,
    ReadLocalRemoteError(String),
    ReadLocalOffline,
    ErrorGetting(String),
}

struct MetadataFetchResult {
    status: GetStatus,
    metadata: Option<CompleteVersionMetadata>,
}

fn get_metadata(
    runtime: &tokio::runtime::Runtime,
    version_info: &VersionInfo,
    data_dir: &Path,
    ctx: &egui::Context,
) -> BackgroundTask<MetadataFetchResult> {
    let version_info = version_info.clone();
    let data_dir = data_dir.to_path_buf();

    let fut = async move {
        let result = CompleteVersionMetadata::read_or_download(&version_info, &data_dir).await;
        match result {
            Ok(metadata) => MetadataFetchResult {
                status: GetStatus::UpToDate,
                metadata: Some(metadata),
            },
            Err(e) => {
                let mut connect_error = false;
                if let Some(re) = e.downcast_ref::<reqwest::Error>() {
                    if re.is_connect() {
                        connect_error = true;
                    }
                }

                let local_metadata =
                    CompleteVersionMetadata::read_local(&version_info, &data_dir).await;
                MetadataFetchResult {
                    status: if connect_error {
                        GetStatus::ReadLocalOffline
                    } else if let Some(local_error) = local_metadata.as_ref().err() {
                        error!(
                            "Error getting metadata:\n{:#}\nlocal metadata error:\n{:#}",
                            e, local_error
                        );
                        GetStatus::ErrorGetting(e.to_string())
                    } else {
                        error!("Error getting metadata:\n{:#}\n(read local)", e);
                        GetStatus::ReadLocalRemoteError(e.to_string())
                    },
                    metadata: local_metadata.ok(),
                }
            }
        }
    };

    let ctx = ctx.clone();
    BackgroundTask::with_callback(fut, runtime, Box::new(move || ctx.request_repaint()))
}

pub struct MetadataState {
    status: GetStatus,
    get_task: Option<BackgroundTask<MetadataFetchResult>>,
    metadata: Option<Arc<CompleteVersionMetadata>>,
}

impl MetadataState {
    pub fn new() -> Self {
        return MetadataState {
            status: GetStatus::Getting,
            get_task: None,
            metadata: None,
        };
    }

    pub fn set_metadata_task(
        &mut self,
        runtime: &Runtime,
        config: &Config,
        version_info: &VersionInfo,
        ctx: &egui::Context,
    ) {
        let launcher_dir = config.get_launcher_dir();
        self.get_task = Some(get_metadata(runtime, version_info, &launcher_dir, ctx));
    }

    pub fn update(&mut self) -> bool {
        if let Some(task) = self.get_task.as_ref() {
            if task.has_result() {
                let task = self.get_task.take().unwrap();
                let result = task.take_result();
                match result {
                    BackgroundTaskResult::Finished(result) => {
                        self.status = result.status;
                        self.metadata = result.metadata.map(Arc::new);
                    }
                    BackgroundTaskResult::Cancelled => {
                        self.status = GetStatus::Getting;
                    }
                }

                return true;
            }
        }

        false
    }

    pub fn get_version_metadata(&self) -> Option<Arc<CompleteVersionMetadata>> {
        self.metadata.clone()
    }

    pub fn online(&self) -> bool {
        self.status == GetStatus::UpToDate
    }

    pub fn is_getting(&self) -> bool {
        self.get_task.is_some()
    }
}
