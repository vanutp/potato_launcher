use std::sync::{Arc, Mutex};

use shared::paths::get_logs_dir;
use tokio::{process::Child, runtime::Runtime};

use crate::{
    auth::user_info::AuthData, config::runtime_config::Config, lang::LangMessage, launcher::launch,
    version::complete_version_metadata::CompleteVersionMetadata,
};

enum LauncherStatus {
    NotLaunched,
    Running { child: Arc<Mutex<Child>> },
    Error(String),
    ProcessErrorCode(String),
}

pub struct LaunchState {
    status: LauncherStatus,
    force_launch: bool,
    launch_from_start: bool,
    ctx: egui::Context,
}

pub enum ForceLaunchResult {
    NotSelected,
    ForceLaunchSelected,
    CancelSelected,
}

impl LaunchState {
    pub fn new(launch_from_start: bool, ctx: egui::Context) -> Self {
        LaunchState {
            status: LauncherStatus::NotLaunched,
            force_launch: false,
            launch_from_start,
            ctx,
        }
    }

    async fn child_callback(child: Arc<Mutex<Child>>, ctx: egui::Context) {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let result = child.lock().unwrap().try_wait();
            match result {
                Ok(Some(_)) => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.request_repaint();
                    break;
                }
                Ok(None) => {
                }
                Err(_) => {
                    break;
                }
            }
        }
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
                let arc_child = Arc::new(Mutex::new(child));
                if config.hide_launcher_after_launch {
                    self.ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }
                runtime.spawn(Self::child_callback(arc_child.clone(), self.ctx.clone()));
                self.status = LauncherStatus::Running { child: arc_child.clone() };
            }
            Err(e) => {
                self.status = LauncherStatus::Error(e.to_string());
            }
        }
    }

    pub fn update(&mut self) {
        match self.status {
            LauncherStatus::Running { ref mut child } => {
                let result = child.lock().unwrap().try_wait();
                match result {
                    Ok(Some(exit_status)) => {
                        self.ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
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
                    let _ = runtime.block_on(child.lock().unwrap().kill());
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
