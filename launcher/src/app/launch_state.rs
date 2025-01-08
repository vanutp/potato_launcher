use std::sync::Arc;

use shared::paths::get_logs_dir;
use tokio::{process::Child, runtime::Runtime};

use crate::{
    auth::user_info::AuthData, config::runtime_config::Config, lang::LangMessage, launcher::launch,
    version::complete_version_metadata::CompleteVersionMetadata,
};

enum LauncherStatus {
    NotLaunched,
    Running { child: Child },
    Error(String),
    ProcessErrorCode(String),
}

pub struct LaunchState {
    status: LauncherStatus,
    force_launch: bool,
    launch_from_start: bool,
    hide_window: bool,
}

pub enum ForceLaunchResult {
    NotSelected,
    ForceLaunchSelected,
    CancelSelected,
}

impl LaunchState {
    pub fn new(launch_from_start: bool) -> Self {
        LaunchState {
            status: LauncherStatus::NotLaunched,
            force_launch: false,
            launch_from_start,
            hide_window: false,
        }
    }

    pub fn hide_window(&self) -> bool {
        self.hide_window
    }

    fn launch(
        &mut self,
        runtime: &Runtime,
        config: &Config,
        selected_instance: &CompleteVersionMetadata,
        auth_data: &AuthData,
        online: bool,
    ) {
        match runtime.block_on(launch::launch(selected_instance, config, auth_data, online)) {
            Ok(child) => {
                if config.close_launcher_after_launch {
                    self.hide_window = true;
                }
                self.status = LauncherStatus::Running { child };
            }
            Err(e) => {
                self.status = LauncherStatus::Error(e.to_string());
            }
        }
    }

    pub fn update(&mut self) {
        match self.status {
            LauncherStatus::Running { ref mut child } => {
                match child.try_wait() {
                    Ok(Some(exit_status)) => {
                        self.hide_window = false;
                        self.status = if exit_status.success() {
                            LauncherStatus::NotLaunched
                        } else {
                            LauncherStatus::ProcessErrorCode(
                                exit_status.code().unwrap_or(-1).to_string(),
                            )
                        };
                    }
                    Ok(None) => {}
                    Err(e) => {
                        self.status = LauncherStatus::Error(e.to_string());
                    }
                };
            }
            _ => {}
        }
    }

    fn big_button_clicked(ui: &mut egui::Ui, text: &str) -> bool {
        let button_text = egui::RichText::new(text)
            .size(20.0)
            .text_style(egui::TextStyle::Button);
        let button = egui::Button::new(button_text);
        ui.add_sized([ui.available_width(), 50.0], button).clicked()
    }

    pub fn render_ui(
        &mut self,
        runtime: &Runtime,
        ui: &mut egui::Ui,
        config: &mut Config,
        selected_instance: Option<Arc<CompleteVersionMetadata>>,
        auth_data: Option<AuthData>,
        online: bool,
        disabled: bool,
    ) {
        let lang = config.lang;

        match &mut self.status {
            LauncherStatus::Running { child } => {
                ui.label(LangMessage::Running.to_string(lang));
                if ui
                    .button(LangMessage::KillMinecraft.to_string(lang))
                    .clicked()
                {
                    let _ = runtime.block_on(child.kill());
                }
            }
            _ => {
                let button_text = if online {
                    LangMessage::Launch.to_string(lang)
                } else {
                    format!(
                        "{} ({})",
                        LangMessage::Launch.to_string(lang),
                        LangMessage::Offline.to_string(lang)
                    )
                };
                let enabled = selected_instance.is_some() && auth_data.is_some() && !disabled;
                ui.add_enabled_ui(enabled, |ui| {
                    if Self::big_button_clicked(ui, &button_text)
                        || (enabled && (self.force_launch || self.launch_from_start))
                    {
                        self.launch_from_start = false;

                        self.force_launch = false;
                        self.launch(
                            runtime,
                            config,
                            &selected_instance.unwrap(),
                            &auth_data.unwrap(),
                            online,
                        );
                    }
                });
            }
        }

        match &self.status {
            LauncherStatus::Error(e) => {
                ui.label(LangMessage::LaunchError(e.clone()).to_string(lang));
            }
            LauncherStatus::ProcessErrorCode(e) => {
                ui.label(LangMessage::ProcessErrorCode(e.clone()).to_string(lang));
                if ui.button(LangMessage::OpenLogs.to_string(lang)).clicked() {
                    open::that(get_logs_dir(&config.get_launcher_dir())).unwrap();
                }
            }
            _ => {}
        }
    }

    pub fn render_download_ui(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut Config,
        disabled: bool,
    ) -> ForceLaunchResult {
        let lang = config.lang;

        if !self.force_launch {
            let mut button_clicked = false;
            ui.add_enabled_ui(!disabled, |ui| {
                if LaunchState::big_button_clicked(
                    ui,
                    &LangMessage::DownloadAndLaunch.to_string(lang),
                ) || (!disabled && self.launch_from_start)
                {
                    self.launch_from_start = false;

                    button_clicked = true;
                }
            });
            if button_clicked {
                self.force_launch = true;
                return ForceLaunchResult::ForceLaunchSelected;
            }
        } else {
            let mut cancel_clicked = false;
            ui.add_enabled_ui(!disabled, |ui| {
                if LaunchState::big_button_clicked(ui, &LangMessage::CancelLaunch.to_string(lang)) {
                    self.force_launch = false;
                    cancel_clicked = true;
                }
            });
            if cancel_clicked {
                return ForceLaunchResult::CancelSelected;
            }
        }
        ForceLaunchResult::NotSelected
    }
}

impl Drop for LaunchState {
    fn drop(&mut self) {}
}
