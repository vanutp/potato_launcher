use shared::paths::get_java_dir;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::config::runtime_config;
use crate::lang::{Lang, LangMessage};
use crate::utils;
use crate::version::complete_version_metadata::CompleteVersionMetadata;

use shared::java;
use shared::progress::{ProgressBar, Unit};

use super::background_task::{BackgroundTask, BackgroundTaskResult};
use super::progress_bar::GuiProgressBar;

#[derive(Clone, PartialEq)]
pub enum JavaDownloadStatus {
    CheckingJava,
    NotDownloaded,
    Downloaded,
    DownloadError(String),
    DownloadErrorOffline,
}

struct JavaCheckResult {
    java_path: Option<PathBuf>,
}

fn check_java(
    runtime: &Runtime,
    java_version: &str,
    java_dir: &Path,
    existing_path: Option<&str>,
    ctx: &egui::Context,
) -> BackgroundTask<JavaCheckResult> {
    let java_version = java_version.to_string();
    let java_dir = java_dir.to_path_buf();
    let existing_path = existing_path.map(|s| s.to_string());
    let ctx = ctx.clone();

    let fut = async move {
        if let Some(path) = existing_path {
            let path = PathBuf::from(path);
            if java::check_java(&java_version, &path).await {
                return JavaCheckResult {
                    java_path: Some(path),
                };
            }
        }
        let java_path = java::get_java(&java_version, &java_dir)
            .await
            .map(|j| j.path);
        JavaCheckResult { java_path }
    };

    BackgroundTask::with_callback(
        fut,
        runtime,
        Box::new(move || {
            ctx.request_repaint();
        }),
    )
}

struct JavaDownloadResult {
    pub status: JavaDownloadStatus,
    pub java_installation: Option<java::JavaInstallation>,
}

fn download_java(
    runtime: &Runtime,
    required_version: &str,
    java_dir: &Path,
    progress_bar: Arc<dyn ProgressBar<LangMessage>>,
) -> BackgroundTask<JavaDownloadResult> {
    let progress_bar_clone = progress_bar.clone();
    let required_version = required_version.to_string();
    let java_dir = java_dir.to_path_buf();
    let fut = async move {
        progress_bar_clone.set_message(LangMessage::DownloadingJava);
        let result = java::download_java(&required_version, &java_dir, progress_bar_clone).await;
        match result {
            Ok(java_installation) => JavaDownloadResult {
                status: JavaDownloadStatus::Downloaded,
                java_installation: Some(java_installation),
            },
            Err(e) => JavaDownloadResult {
                status: if utils::is_connect_error(&e) {
                    JavaDownloadStatus::DownloadErrorOffline
                } else {
                    JavaDownloadStatus::DownloadError(e.to_string())
                },
                java_installation: None,
            },
        }
    };

    BackgroundTask::with_callback(
        fut,
        runtime,
        Box::new(move || {
            progress_bar.finish();
        }),
    )
}

pub struct JavaState {
    status: JavaDownloadStatus,
    check_java_task: Option<BackgroundTask<JavaCheckResult>>,
    java_download_task: Option<BackgroundTask<JavaDownloadResult>>,
    java_download_progress_bar: Arc<GuiProgressBar>,
    settings_opened: bool,
}

impl JavaState {
    pub fn new(ctx: &egui::Context) -> Self {
        let java_download_progress_bar = Arc::new(GuiProgressBar::new(ctx));
        java_download_progress_bar.set_unit(Unit {
            name: "MB".to_string(),
            size: 1024 * 1024,
        });
        Self {
            status: JavaDownloadStatus::CheckingJava,
            check_java_task: None,
            java_download_task: None,
            java_download_progress_bar,
            settings_opened: false,
        }
    }

    fn schedule_download(
        &mut self,
        runtime: &Runtime,
        metadata: &CompleteVersionMetadata,
        config: &mut runtime_config::Config,
    ) {
        let launcher_dir = config.get_launcher_dir();
        let java_dir = get_java_dir(&launcher_dir);

        self.java_download_progress_bar.reset();

        self.java_download_task = Some(download_java(
            runtime,
            &metadata.get_java_version(),
            &java_dir,
            self.java_download_progress_bar.clone(),
        ));
    }

    pub fn update(
        &mut self,
        runtime: &Runtime,
        metadata: &CompleteVersionMetadata,
        config: &mut runtime_config::Config,
        ctx: &egui::Context,
        need_java_check: bool,
    ) {
        if need_java_check {
            self.status = JavaDownloadStatus::CheckingJava;
            let launcher_dir = config.get_launcher_dir();
            let java_dir = get_java_dir(&launcher_dir);

            self.check_java_task = Some(check_java(
                runtime,
                &metadata.get_java_version(),
                &java_dir,
                config
                    .java_paths
                    .get(metadata.get_name())
                    .map(|s| s.as_str()),
                ctx,
            ));

            self.settings_opened = false;
        }

        if let Some(task) = self.check_java_task.as_ref() {
            if task.has_result() {
                let task = self.check_java_task.take().unwrap();
                let result = task.take_result();

                match result {
                    BackgroundTaskResult::Finished(result) => {
                        if let Some(java_path) = result.java_path {
                            config.java_paths.insert(
                                metadata.get_name().to_string(),
                                java_path.to_string_lossy().to_string(),
                            );
                            config.save();
                            self.status = JavaDownloadStatus::Downloaded;
                        } else {
                            config.java_paths.remove(metadata.get_name());
                            config.save();
                            self.status = JavaDownloadStatus::NotDownloaded;
                        }
                    }

                    BackgroundTaskResult::Cancelled => {
                        self.status = JavaDownloadStatus::NotDownloaded;
                    }
                }
            }
        }

        if let Some(task) = self.java_download_task.as_ref() {
            if task.has_result() {
                let task = self.java_download_task.take().unwrap();
                let result = task.take_result();

                match result {
                    BackgroundTaskResult::Finished(result) => {
                        self.status = result.status;
                        if self.status == JavaDownloadStatus::Downloaded {
                            let path = result.java_installation.as_ref().unwrap().path.clone();
                            config.java_paths.insert(
                                metadata.get_name().to_string(),
                                path.to_string_lossy().to_string(),
                            );
                            config.save();
                        }
                    }
                    BackgroundTaskResult::Cancelled => {
                        self.status = JavaDownloadStatus::NotDownloaded;
                    }
                }
            }
        }
    }

    fn is_download_needed(&self) -> bool {
        if self.java_download_task.is_some() {
            return false;
        }
        match self.status {
            JavaDownloadStatus::CheckingJava => false,
            JavaDownloadStatus::NotDownloaded => true,
            JavaDownloadStatus::DownloadError(_) => true,
            JavaDownloadStatus::DownloadErrorOffline => true,
            JavaDownloadStatus::Downloaded => false,
        }
    }

    pub fn schedule_download_if_needed(
        &mut self,
        runtime: &Runtime,
        metadata: &CompleteVersionMetadata,
        config: &mut runtime_config::Config,
    ) {
        if self.is_download_needed() {
            self.schedule_download(runtime, metadata, config);
        }
    }

    pub fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut runtime_config::Config,
        selected_metadata: &CompleteVersionMetadata,
    ) {
        match self.status {
            JavaDownloadStatus::CheckingJava => {
                ui.label(LangMessage::CheckingJava.to_string(&config.lang));
            }
            JavaDownloadStatus::NotDownloaded => {
                if self.java_download_task.is_none() {
                    ui.label(
                        LangMessage::NeedJava {
                            version: selected_metadata.get_java_version().clone(),
                        }
                        .to_string(&config.lang),
                    );
                }
            }
            JavaDownloadStatus::DownloadError(ref e) => {
                ui.label(LangMessage::ErrorDownloadingJava(e.clone()).to_string(&config.lang));
            }
            JavaDownloadStatus::DownloadErrorOffline => {
                ui.label(LangMessage::NoConnectionToJavaServer.to_string(&config.lang));
            }
            JavaDownloadStatus::Downloaded => {
                ui.label(
                    LangMessage::JavaInstalled {
                        version: selected_metadata.get_java_version().clone(),
                    }
                    .to_string(&config.lang),
                );
            }
        }

        if self.java_download_task.is_some() {
            self.java_download_progress_bar.render(ui, &config.lang);
            self.render_cancel_button(ui, &config.lang);
        }
    }

    pub fn ready_for_launch(&self) -> bool {
        self.status == JavaDownloadStatus::Downloaded
    }

    fn render_cancel_button(&mut self, ui: &mut egui::Ui, lang: &Lang) {
        if ui
            .button(LangMessage::CancelDownload.to_string(lang))
            .clicked()
        {
            self.cancel_download();
        }
    }

    pub fn cancel_download(&mut self) {
        if let Some(task) = self.java_download_task.as_ref() {
            task.cancel();
        }
    }
}
