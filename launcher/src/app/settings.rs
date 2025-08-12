use super::language_selector::LanguageSelector;
use crate::config::build_config::USE_NATIVE_GLFW_DEFAULT;
use crate::config::runtime_config::Config;
use crate::constants::{XMX_DEFAULT, XMX_MAX, XMX_MIN, XMX_STEP};
use crate::lang::LangMessage;
use crate::utils;
use crate::version::complete_version_metadata::CompleteVersionMetadata;
use shared::java;
use tokio::runtime::Runtime;

fn get_xmx_max() -> f64 {
    utils::get_total_memory().map_or(XMX_MAX, |total| total / 1024) as f64
}

pub struct SettingsState {
    language_selector: LanguageSelector,
    settings_opened: bool,
    instance_settings_opened: bool,
    picked_java_path: Option<String>,
    xmx_slider_value: f64,
    use_native_glfw: bool,
}

fn map_xmx_slider_value(value: f64) -> String {
    let mb = utils::map_range(value, 0.0, 1.0, XMX_MIN as f64, get_xmx_max()) as u64;
    format!("{}M", ((mb + XMX_STEP / 2) / XMX_STEP) * XMX_STEP)
}

fn map_xmx_slider_value_reverse(value: &str) -> f64 {
    let xmx = value
        .trim_end_matches('M')
        .parse::<u64>()
        .unwrap_or(XMX_DEFAULT);
    utils::map_range(xmx as f64, XMX_MIN as f64, get_xmx_max(), 0.0, 1.0)
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            language_selector: LanguageSelector::new(),
            settings_opened: false,
            instance_settings_opened: false,
            picked_java_path: None,
            xmx_slider_value: 0.0,
            use_native_glfw: false,
        }
    }

    pub fn render_settings(&mut self, ui: &mut egui::Ui, config: &mut Config) {
        if ui.button("📂").clicked() {
            open::that(config.get_launcher_dir()).unwrap();
        }

        if ui
            .add_enabled(!self.settings_opened, egui::Button::new("⚙"))
            .clicked()
        {
            self.settings_opened = true;
        }

        self.language_selector.render_ui(ui, config);

        self.render_settings_window(ui, config);
    }

    pub fn render_settings_window(&mut self, ui: &mut egui::Ui, config: &mut Config) {
        let lang = config.lang;
        let mut settings_opened = self.settings_opened;

        egui::Window::new(LangMessage::Settings.to_string(lang))
            .open(&mut settings_opened)
            .show(ui.ctx(), |ui| {
                self.render_close_launcher_checkbox(ui, config);
            });

        self.settings_opened = settings_opened;
    }

    pub fn render_instance_settings(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        config: &mut Config,
        selected_metadata: Option<&CompleteVersionMetadata>,
    ) {
        if ui
            .add_enabled(
                selected_metadata.is_some() && !self.instance_settings_opened,
                egui::Button::new("⚙"),
            )
            .clicked()
        {
            self.instance_settings_opened = true;
            let selected_metadata = selected_metadata.unwrap();
            self.picked_java_path = config.java_paths.get(selected_metadata.get_name()).cloned();
            self.xmx_slider_value = map_xmx_slider_value_reverse(
                config
                    .xmx
                    .get(selected_metadata.get_name())
                    .unwrap_or(&XMX_DEFAULT.to_string()),
            );
            self.use_native_glfw = *config
                .use_native_glfw
                .get(selected_metadata.get_name())
                .unwrap_or(&USE_NATIVE_GLFW_DEFAULT);
        }

        if let Some(selected_metadata) = selected_metadata {
            self.render_instance_settings_window(ui, runtime, config, selected_metadata);
        } else {
            self.instance_settings_opened = false;
        }
    }

    #[cfg(target_os = "linux")]
    fn render_use_native_glfw_checkbox(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut Config,
        selected_metadata: &CompleteVersionMetadata,
    ) {
        let old_use_native_glfw = self.use_native_glfw;
        ui.checkbox(
            &mut self.use_native_glfw,
            LangMessage::UseNativeGlfw.to_string(config.lang),
        );
        if old_use_native_glfw != self.use_native_glfw {
            config.use_native_glfw.insert(
                selected_metadata.get_name().to_string(),
                self.use_native_glfw,
            );
            config.save();
        }
    }

    fn render_instance_settings_window(
        &mut self,
        ui: &mut egui::Ui,
        runtime: &Runtime,
        config: &mut Config,
        selected_metadata: &CompleteVersionMetadata,
    ) {
        let lang = config.lang;
        let mut settings_opened = self.instance_settings_opened;

        egui::Window::new(LangMessage::InstanceSettings.to_string(lang))
            .open(&mut settings_opened)
            .show(ui.ctx(), |ui| {
                if let Some(picked_java_path) = &self.picked_java_path {
                    ui.label(LangMessage::SelectedJavaPath.to_string(lang));
                    ui.code(picked_java_path);
                } else {
                    ui.label(LangMessage::NoJavaPath.to_string(lang));
                }

                if ui
                    .button(LangMessage::SelectJavaPath.to_string(lang))
                    .clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                {
                    if runtime.block_on(java::check_java(
                        &selected_metadata.get_java_version(),
                        &path,
                    )) {
                        self.picked_java_path = Some(path.display().to_string());
                        config.java_paths.insert(
                            selected_metadata.get_name().to_string(),
                            path.display().to_string(),
                        );
                        config.save();
                    } else {
                        self.picked_java_path =
                            LangMessage::InvalidJavaInstallation.to_string(lang).into();
                    }
                }

                ui.label(LangMessage::AllocatedMemory.to_string(lang));
                let old_xmx = self.xmx_slider_value;
                let xmx_slider = egui::Slider::new(&mut self.xmx_slider_value, 0.0..=1.0)
                    .custom_formatter(|value, _| {
                        format!(
                            "{}M",
                            ((utils::map_range(value, 0.0, 1.0, XMX_MIN as f64, get_xmx_max())
                                as u64
                                + XMX_STEP / 2)
                                / XMX_STEP)
                                * XMX_STEP
                        )
                    })
                    .custom_parser(|value| {
                        let mb = utils::format_xmx(Some(value))
                            .trim_end_matches('M')
                            .parse::<u64>()
                            .unwrap_or(XMX_DEFAULT);
                        let value_unclamped =
                            utils::map_range(mb as f64, XMX_MIN as f64, get_xmx_max(), 0.0, 1.0);
                        Some(value_unclamped.clamp(0.0, 1.0))
                    });
                ui.add(xmx_slider);
                if old_xmx != self.xmx_slider_value {
                    config.xmx.insert(
                        selected_metadata.get_name().to_string(),
                        map_xmx_slider_value(self.xmx_slider_value),
                    );
                    config.save();
                }

                #[cfg(target_os = "linux")]
                self.render_use_native_glfw_checkbox(ui, config, selected_metadata);
            });

        self.instance_settings_opened = settings_opened;
    }

    fn render_close_launcher_checkbox(&mut self, ui: &mut egui::Ui, config: &mut Config) {
        let old_close_launcher_after_launch = config.hide_launcher_after_launch;
        ui.checkbox(
            &mut config.hide_launcher_after_launch,
            LangMessage::HideLauncherAfterLaunch.to_string(config.lang),
        );
        if old_close_launcher_after_launch != config.hide_launcher_after_launch {
            config.save();
        }
    }
}
