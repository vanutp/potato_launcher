use image::Luma;
use qrcode::QrCode;
use shared::version::extra_version_metadata::AuthData;
use std::io::Cursor;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::auth::auth::auth; // nice
use crate::auth::auth::AuthMessageProvider;
use crate::auth::base::get_auth_provider;
use crate::auth::version_auth_data::AuthStorage;
use crate::auth::version_auth_data::RuntimeAuthStorage;
use crate::auth::version_auth_data::VersionAuthData;
use crate::config::runtime_config::Config;
use crate::lang::{Lang, LangMessage};
use crate::message_provider::MessageProvider as _;

use super::background_task::{BackgroundTask, BackgroundTaskResult};

#[derive(Clone, PartialEq)]
enum AuthStatus {
    NotAuthorized,
    Authorized,
    AuthorizeError(String),
    AuthorizeErrorOffline,
    AuthorizeErrorTimeout,
}

struct AuthResult {
    auth_data: AuthData,
    status: AuthStatus,
    version_auth_data: Option<VersionAuthData>,
}

fn authenticate(
    runtime: &Runtime,
    version_auth_data: Option<&VersionAuthData>,
    auth_data: &AuthData,
    auth_message_provider: Arc<AuthMessageProvider>,
    ctx: &egui::Context,
) -> BackgroundTask<AuthResult> {
    let ctx = ctx.clone();
    let auth_data = auth_data.clone();
    let auth_provider = get_auth_provider(&auth_data);
    let version_auth_data = version_auth_data.cloned();

    let fut = async move {
        match auth(version_auth_data, auth_provider, auth_message_provider).await {
            Ok(data) => AuthResult {
                auth_data: auth_data,
                status: AuthStatus::Authorized,
                version_auth_data: Some(data),
            },

            Err(e) => {
                let mut connect_error = false;
                let mut timeout_error = false;
                if let Some(re) = e.downcast_ref::<reqwest::Error>() {
                    if re.is_connect() {
                        connect_error = true;
                    }
                    if re.is_timeout() || re.status().map(|s| s.as_u16()) == Some(524) {
                        timeout_error = true;
                    }
                }

                AuthResult {
                    auth_data: auth_data,
                    status: if connect_error {
                        AuthStatus::AuthorizeErrorOffline
                    } else if timeout_error {
                        AuthStatus::AuthorizeErrorTimeout
                    } else {
                        AuthStatus::AuthorizeError(e.to_string())
                    },
                    version_auth_data: None,
                }
            }
        }
    };

    BackgroundTask::with_callback(
        fut,
        runtime,
        Box::new(move || {
            ctx.request_repaint();
        }),
    )
}

pub struct AuthState {
    auth_status: AuthStatus,
    auth_task: Option<BackgroundTask<AuthResult>>,
    auth_message_provider: Arc<AuthMessageProvider>,
    auth_storage: AuthStorage,
    runtime_auth_storage: RuntimeAuthStorage,
}

impl AuthState {
    pub fn new(ctx: &egui::Context) -> Self {
        return AuthState {
            auth_status: AuthStatus::NotAuthorized,
            auth_task: None,
            auth_message_provider: Arc::new(AuthMessageProvider::new(ctx)),
            auth_storage: AuthStorage::load(),
            runtime_auth_storage: RuntimeAuthStorage::new(),
        };
    }

    pub fn update(&mut self) -> bool {
        if let Some(task) = self.auth_task.as_ref() {
            if task.has_result() {
                self.auth_message_provider.clear();
                let task = self.auth_task.take().unwrap();
                let result = task.take_result();
                match result {
                    BackgroundTaskResult::Finished(result) => {
                        match &result.status {
                            AuthStatus::Authorized => {
                                if let Some(version_auth_data) = result.version_auth_data {
                                    self.runtime_auth_storage
                                        .insert(&result.auth_data, version_auth_data.clone());
                                    let _ = self
                                        .auth_storage
                                        .insert(&result.auth_data, version_auth_data);
                                }
                            }
                            _ => {}
                        }

                        self.auth_status = result.status;
                    }

                    BackgroundTaskResult::Cancelled => {
                        self.auth_status = AuthStatus::NotAuthorized;
                    }
                }

                return true;
            }
        }

        false
    }

    fn render_auth_window(auth_message: LangMessage, lang: Lang, ui: &mut egui::Ui) {
        egui::Window::new(LangMessage::Authorization.to_string(lang)).show(ui.ctx(), |ui| {
            ui.label(auth_message.to_string(lang));
            let url = match auth_message {
                LangMessage::AuthMessage { url } => Some(url),
                LangMessage::DeviceAuthMessage { url, .. } => Some(url),
                _ => None,
            }
            .unwrap();

            ui.hyperlink(&url);
            let code = QrCode::new(url).unwrap();
            let image = code.render::<Luma<u8>>().build();

            let mut png_bytes: Vec<u8> = Vec::new();
            let mut cursor = Cursor::new(&mut png_bytes);
            image::DynamicImage::ImageLuma8(image)
                .write_to(&mut cursor, image::ImageFormat::Png)
                .unwrap();

            let uri = "bytes://auth_qr.png";
            ui.ctx().include_bytes(uri, png_bytes.clone());
            ui.add(egui::Image::from_bytes(uri.to_string(), png_bytes));
        });
    }

    pub fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        config: &Config,
        runtime: &Runtime,
        ctx: &egui::Context,
        auth_data: &AuthData,
    ) {
        let lang = config.lang;
        let version_auth_data = self.auth_storage.get(auth_data);
        let selected_username = version_auth_data.map(|x| x.user_info.username.clone());

        let auth_provider = get_auth_provider(auth_data);
        let auth_provider_name = auth_provider.get_name();

        match &self.auth_status {
            AuthStatus::NotAuthorized if self.auth_task.is_none() => {
                if let Some(version_auth_data) = self.auth_storage.get(auth_data) {
                    self.auth_message_provider = Arc::new(AuthMessageProvider::new(&ctx));
                    self.auth_task = Some(authenticate(
                        runtime,
                        Some(version_auth_data),
                        auth_data,
                        self.auth_message_provider.clone(),
                        ctx,
                    ));
                }
            }
            _ => {}
        }

        match &self.auth_status {
            AuthStatus::NotAuthorized if self.auth_task.is_none() => {
                ui.label(LangMessage::AuthorizeUsing(auth_provider_name).to_string(lang));
            }
            AuthStatus::NotAuthorized => {
                ui.label(LangMessage::Authorizing.to_string(lang));
            }
            AuthStatus::AuthorizeError(e) => {
                ui.label(LangMessage::AuthError(e.clone()).to_string(lang));
            }
            AuthStatus::AuthorizeErrorOffline => {
                ui.label(
                    LangMessage::NoConnectionToAuthServer {
                        offline_username: selected_username.clone(),
                    }
                    .to_string(lang),
                );
            }
            AuthStatus::AuthorizeErrorTimeout => {
                ui.label(LangMessage::AuthTimeout.to_string(lang));
            }
            AuthStatus::Authorized => {
                if let Some(version_auth_data) = version_auth_data {
                    ui.label(LangMessage::AuthorizedAs.to_string(lang));
                    let text = egui::RichText::new(&version_auth_data.user_info.username)
                        .text_style(egui::TextStyle::Monospace);
                    ui.label(text);
                } else {
                    ui.label(LangMessage::LogicError.to_string(lang));
                }
            }
        }

        if let Some(message) = self.auth_message_provider.get_message() {
            AuthState::render_auth_window(message, lang, ui);
        }

        match &self.auth_status {
            AuthStatus::Authorized => {}
            AuthStatus::NotAuthorized if self.auth_task.is_some() => {}
            _ => {
                if ui.button(LangMessage::Authorize.to_string(lang)).clicked() {
                    self.auth_message_provider = Arc::new(AuthMessageProvider::new(&ctx));
                    self.auth_task = Some(authenticate(
                        runtime,
                        version_auth_data,
                        auth_data,
                        self.auth_message_provider.clone(),
                        ctx,
                    ));
                }
            }
        }
    }

    pub fn get_version_auth_data(&self, auth_data: &AuthData) -> Option<&VersionAuthData> {
        if let Some(version_auth_data) = self.runtime_auth_storage.get(auth_data) {
            return Some(version_auth_data);
        }
        if self.auth_status == AuthStatus::AuthorizeErrorOffline {
            if let Some(version_auth_data) = self.auth_storage.get(auth_data) {
                return Some(version_auth_data);
            }
        }
        None
    }

    pub fn reset_auth_if_needed(&mut self, new_auth_data: &AuthData) {
        if !self.runtime_auth_storage.contains(new_auth_data) {
            self.auth_status = AuthStatus::NotAuthorized;
            self.auth_task = None;
        }
    }

    pub fn online(&self) -> bool {
        match &self.auth_status {
            AuthStatus::Authorized => true,
            _ => false,
        }
    }
}
