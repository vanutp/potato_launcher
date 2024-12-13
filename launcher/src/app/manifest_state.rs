use crate::{
    config::{build_config, runtime_config::Config},
    lang::LangMessage,
};

use shared::version::version_manifest::VersionManifest;
use tokio::runtime::Runtime;

use super::background_task::{BackgroundTask, BackgroundTaskResult};

#[derive(PartialEq)]
enum FetchStatus {
    NotFetched,
    Fetched,
    FetchErrorOffline,
    FetchError(String),
}

struct ManifestFetchResult {
    status: FetchStatus,
    manifest: Option<VersionManifest>,
}

fn fetch_manifest<Callback>(
    runtime: &tokio::runtime::Runtime,
    callback: Callback,
) -> BackgroundTask<ManifestFetchResult>
where
    Callback: FnOnce() + Send + 'static,
{
    let fut = async move {
        let result = VersionManifest::fetch(&build_config::get_version_manifest_url()).await;
        match result {
            Ok(manifest) => ManifestFetchResult {
                status: FetchStatus::Fetched,
                manifest: Some(manifest),
            },
            Err(e) => {
                let mut connect_error = false;
                if let Some(re) = e.downcast_ref::<reqwest::Error>() {
                    if re.is_connect() {
                        connect_error = true;
                    }
                }

                ManifestFetchResult {
                    status: if connect_error {
                        FetchStatus::FetchErrorOffline
                    } else {
                        FetchStatus::FetchError(e.to_string())
                    },
                    manifest: None,
                }
            }
        }
    };

    BackgroundTask::with_callback(fut, runtime, Box::new(callback))
}

pub struct ManifestState {
    status: FetchStatus,
    fetch_task: Option<BackgroundTask<ManifestFetchResult>>,
}

impl ManifestState {
    fn set_fetch_task(&mut self, runtime: &Runtime, ctx: &egui::Context) {
        let ctx = ctx.clone();
        self.fetch_task = Some(fetch_manifest(runtime, move || {
            ctx.request_repaint();
        }));
    }

    pub fn new(runtime: &Runtime, ctx: &egui::Context) -> ManifestState {
        let mut result = ManifestState {
            status: FetchStatus::NotFetched,
            fetch_task: None,
        };
        result.set_fetch_task(runtime, ctx);

        result
    }

    pub fn take_manifest(&mut self, config: &mut Config) -> Option<VersionManifest> {
        if let Some(task) = self.fetch_task.as_ref() {
            if task.has_result() {
                let task = self.fetch_task.take().unwrap();
                let result = task.take_result();
                match result {
                    BackgroundTaskResult::Finished(result) => {
                        if let Some(manifest) = &result.manifest {
                            if config.selected_instance_name.is_none()
                                && manifest.versions.len() == 1
                            {
                                config.selected_instance_name =
                                    manifest.versions.first().map(|x| x.get_name());
                                config.save();
                            }
                        }
                        self.status = result.status;

                        return result.manifest;
                    }
                    BackgroundTaskResult::Cancelled => {
                        self.status = FetchStatus::NotFetched;
                    }
                }
            }
        }

        None
    }

    pub fn render_combo_box(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut Config,
        local_instance_names: &Vec<String>,
        remote_instance_names: &Vec<String>,
    ) -> bool {
        let mut selected_instance_name = config.selected_instance_name.clone();

        ui.horizontal(|ui| {
            ui.label(LangMessage::SelectInstance.to_string(config.lang));
            egui::ComboBox::from_id_source("instances")
                .selected_text(
                    selected_instance_name
                        .clone()
                        .unwrap_or_else(|| LangMessage::NotSelected.to_string(config.lang)),
                )
                .show_ui(ui, |ui| {
                    if !local_instance_names.is_empty() || !remote_instance_names.is_empty() {
                        for instance_name in local_instance_names {
                            ui.selectable_value(
                                &mut selected_instance_name,
                                Some(instance_name.clone()),
                                instance_name,
                            );
                        }
                        for instance_name in remote_instance_names {
                            ui.selectable_value(
                                &mut selected_instance_name,
                                Some(instance_name.clone()),
                                egui::WidgetText::from(instance_name).italics(),
                            );
                        }
                    } else {
                        ui.label(LangMessage::NoInstances.to_string(config.lang));
                    }
                });
        });

        if config.selected_instance_name != selected_instance_name {
            config.selected_instance_name = selected_instance_name;
            config.save();
            true
        } else {
            false
        }
    }

    pub fn render_status(&mut self, runtime: &Runtime, ui: &mut egui::Ui, config: &Config) {
        let lang = config.lang;

        match self.status {
            FetchStatus::NotFetched => {
                ui.label(LangMessage::FetchingVersionManifest.to_string(lang));
            }
            FetchStatus::Fetched => {}
            FetchStatus::FetchErrorOffline => {
                ui.label(LangMessage::NoConnectionToManifestServer.to_string(lang));
            }
            FetchStatus::FetchError(ref s) => {
                ui.label(LangMessage::ErrorFetchingRemoteManifest(s.clone()).to_string(lang));
            }
        }

        if self.status != FetchStatus::Fetched && self.status != FetchStatus::NotFetched {
            if ui
                .button(LangMessage::FetchManifest.to_string(lang))
                .clicked()
            {
                self.status = FetchStatus::NotFetched;
                self.set_fetch_task(&runtime, ui.ctx());
            }
        }
    }

    pub fn online(&self) -> bool {
        self.status == FetchStatus::Fetched
    }
}
