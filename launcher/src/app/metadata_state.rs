use std::{collections::HashMap, path::Path, sync::Arc};

use egui::RichText;
use log::{error, info};
use shared::utils::is_connect_error;
use shared::version::version_manifest::VersionInfo;
use tokio::runtime::Runtime;

use crate::{
    config::runtime_config::Config, lang::LangMessage,
    version::complete_version_metadata::CompleteVersionMetadata,
};

use super::{
    background_task::{BackgroundTask, BackgroundTaskResult},
    colors,
};

#[derive(PartialEq)]
enum GetStatus {
    NoMetadata,
    UpToDate,
    ReadLocalRemoteError,
    ReadLocalOffline,
    ErrorGetting,
}

struct MetadataFetchResult {
    status: GetStatus,
    version_info: VersionInfo,
    metadata: Option<Arc<CompleteVersionMetadata>>,
}

fn get_metadata(
    runtime: &tokio::runtime::Runtime,
    version_info: &VersionInfo,
    data_dir: &Path,
    ctx: &egui::Context,
    existing_metadata: Option<Arc<CompleteVersionMetadata>>,
) -> BackgroundTask<MetadataFetchResult> {
    let version_info = version_info.clone();
    let data_dir = data_dir.to_path_buf();

    let fut = async move {
        if let Some(metadata) = existing_metadata {
            return MetadataFetchResult {
                status: GetStatus::UpToDate,
                version_info,
                metadata: Some(metadata),
            };
        }
        let result = CompleteVersionMetadata::read_or_download(&version_info, &data_dir).await;
        match result {
            Ok(metadata) => MetadataFetchResult {
                status: GetStatus::UpToDate,
                version_info,
                metadata: Some(Arc::new(metadata)),
            },
            Err(e) => {
                let local_metadata =
                    CompleteVersionMetadata::read_local(&version_info, &data_dir).await;
                MetadataFetchResult {
                    status: if is_connect_error(&e) {
                        info!("Metadata offline mode");
                        GetStatus::ReadLocalOffline
                    } else if let Some(local_error) = local_metadata.as_ref().err() {
                        error!(
                            "Error getting metadata:\n{e:?}\nlocal metadata error:\n{local_error:?}"
                        );
                        GetStatus::ErrorGetting
                    } else {
                        error!("Error getting metadata (reading local instead):\n{e:?}");
                        GetStatus::ReadLocalRemoteError
                    },
                    version_info,
                    metadata: local_metadata.ok().map(Arc::new),
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
    metadata_storage: HashMap<String, Arc<CompleteVersionMetadata>>,
}

impl MetadataState {
    pub fn new() -> Self {
        MetadataState {
            status: GetStatus::NoMetadata,
            get_task: None,
            metadata_storage: HashMap::new(),
        }
    }

    pub fn set_metadata_task(
        &mut self,
        runtime: &Runtime,
        config: &Config,
        version_info: &VersionInfo,
        ctx: &egui::Context,
    ) {
        self.status = GetStatus::NoMetadata;
        let name = version_info.get_name();
        let existing_metadata = self.metadata_storage.get(&name).cloned();
        let launcher_dir = config.get_launcher_dir();
        self.get_task = Some(get_metadata(
            runtime,
            version_info,
            &launcher_dir,
            ctx,
            existing_metadata,
        ));
    }

    pub fn render_status(&mut self, ui: &mut egui::Ui, config: &Config) -> bool {
        let dark_mode = ui.style().visuals.dark_mode;

        ui.label(match self.status {
            GetStatus::NoMetadata => RichText::new(
                if self.get_task.is_some() {
                    LangMessage::GettingMetadata
                } else {
                    LangMessage::NoMetadata
                }
                .to_string(config.lang),
            )
            .color(colors::in_progress(dark_mode)),
            GetStatus::UpToDate => {
                return false;
            }
            GetStatus::ReadLocalRemoteError => {
                RichText::new(LangMessage::ReadLocalRemoteError.to_string(config.lang))
                    .color(colors::partial_error(dark_mode))
            }
            GetStatus::ReadLocalOffline => {
                RichText::new(LangMessage::ReadLocalOffline.to_string(config.lang))
                    .color(colors::offline(dark_mode))
            }
            GetStatus::ErrorGetting => {
                RichText::new(LangMessage::ErrorGettingMetadata.to_string(config.lang))
                    .color(colors::error(dark_mode))
            }
        });

        true
    }

    pub fn update(&mut self) -> bool {
        if let Some(task) = self.get_task.as_ref()
            && task.has_result()
        {
            let task = self.get_task.take().unwrap();
            let result = task.take_result();
            match result {
                BackgroundTaskResult::Finished(result) => {
                    self.status = result.status;
                    let name = result.version_info.get_name();
                    if let Some(metadata) = result.metadata {
                        self.metadata_storage.insert(name, metadata);
                    } else {
                        self.metadata_storage.remove(&name);
                    }
                }
                BackgroundTaskResult::Cancelled => {
                    self.status = GetStatus::NoMetadata;
                }
            }

            true
        } else {
            false
        }
    }

    pub fn get_version_metadata(&self, config: &Config) -> Option<Arc<CompleteVersionMetadata>> {
        self.metadata_storage
            .get(config.selected_instance_name.as_ref()?)
            .cloned()
    }

    pub fn online(&self) -> bool {
        self.status == GetStatus::UpToDate
    }

    pub fn is_getting(&self) -> bool {
        self.get_task.is_some()
    }

    pub fn reset(&mut self, clear_storage: bool) {
        self.status = GetStatus::NoMetadata;
        if clear_storage {
            self.metadata_storage.clear();
        }
    }
}
