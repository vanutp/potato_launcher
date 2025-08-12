use egui::ComboBox;
use egui::RichText;
use egui::Window;
use image::Luma;
use log::error;
use qrcode::QrCode;
use shared::utils::is_connect_error;
use shared::version::extra_version_metadata::AuthBackend;
use shared::version::extra_version_metadata::ElyByAuthBackend;
use shared::version::extra_version_metadata::TelegramAuthBackend;
use std::hash::DefaultHasher;
use std::hash::Hash as _;
use std::hash::Hasher as _;
use std::io::Cursor;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::auth::auth_flow::AuthMessageProvider;
use crate::auth::auth_flow::perform_auth;
use crate::auth::auth_storage::AuthDataSource;
use crate::auth::auth_storage::AuthStorage;
use crate::auth::auth_storage::StorageEntry;
use crate::auth::base::get_auth_provider;
use crate::auth::user_info::AuthData;
use crate::config::runtime_config::AuthProfile;
use crate::config::runtime_config::Config;
use crate::lang::{Lang, LangMessage};
use crate::utils::is_valid_minecraft_username;

use super::background_task::{BackgroundTask, BackgroundTaskResult};
use super::colors;

#[derive(Clone, PartialEq)]
enum AuthStatus {
    NotAuthorized,
    Authorized,
    AuthorizeError,
    AuthorizeErrorOffline,
    AuthorizeErrorTimeout,
}

struct AuthResult {
    auth_backend: AuthBackend,
    status: AuthStatus,
    auth_data: Option<AuthData>,
}

fn authenticate(
    runtime: &Runtime,
    auth_data: Option<AuthData>,
    auth_backend: &AuthBackend,
    auth_message_provider: Arc<AuthMessageProvider>,
    ctx: &egui::Context,
) -> BackgroundTask<AuthResult> {
    let ctx = ctx.clone();
    let auth_backend = auth_backend.clone();
    let auth_provider = get_auth_provider(&auth_backend);

    let fut = async move {
        match perform_auth(auth_data, auth_provider, auth_message_provider).await {
            Ok(data) => AuthResult {
                auth_backend,
                status: AuthStatus::Authorized,
                auth_data: Some(data),
            },

            Err(e) => {
                let mut connect_error = false;
                if is_connect_error(&e) {
                    connect_error = true;
                }
                let mut timeout_error = false;
                if let Some(re) = e.downcast_ref::<reqwest::Error>()
                    && (re.is_timeout() || re.status().map(|s| s.as_u16()) == Some(524))
                {
                    timeout_error = true;
                }

                AuthResult {
                    auth_backend,
                    status: if connect_error {
                        AuthStatus::AuthorizeErrorOffline
                    } else if timeout_error {
                        AuthStatus::AuthorizeErrorTimeout
                    } else {
                        error!("Auth error:\n{e:?}");
                        AuthStatus::AuthorizeError
                    },
                    auth_data: None,
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

#[derive(Clone, Copy, PartialEq)]
enum NewAccountType {
    Microsoft,
    ElyBy,
    Telegram,
    Offline,
}

pub struct AuthState {
    auth_status: AuthStatus,
    auth_task: Option<BackgroundTask<AuthResult>>,
    auth_message_provider: Arc<AuthMessageProvider>,
    auth_storage: AuthStorage,

    show_add_account: bool,

    new_account_type: NewAccountType,

    ely_by_client_id: String,
    ely_by_client_secret: String,

    telegram_auth_base_url: String,

    offline_nickname: String,

    last_auth_profile: Option<AuthProfile>,
}

impl AuthState {
    pub fn new(ctx: &egui::Context, config: &Config) -> Self {
        AuthState {
            auth_status: AuthStatus::NotAuthorized,
            auth_task: None,
            auth_message_provider: Arc::new(AuthMessageProvider::new(ctx)),
            auth_storage: AuthStorage::load(config),

            show_add_account: false,

            new_account_type: NewAccountType::Microsoft,

            ely_by_client_id: String::new(),
            ely_by_client_secret: String::new(),

            telegram_auth_base_url: String::new(),

            offline_nickname: String::new(),

            last_auth_profile: None,
        }
    }

    pub fn update(&mut self, runtime: &Runtime, config: &mut Config) -> bool {
        if let Some(task) = self.auth_task.as_ref()
            && task.has_result()
        {
            runtime.block_on(self.auth_message_provider.clear());
            let task = self.auth_task.take().unwrap();
            let result = task.take_result();
            match result {
                BackgroundTaskResult::Finished(result) => {
                    if result.status == AuthStatus::Authorized
                        && let Some(auth_data) = result.auth_data
                    {
                        config.set_selected_auth_profile(AuthProfile {
                            auth_backend_id: result.auth_backend.get_id(),
                            username: auth_data.user_info.username.clone(),
                        });

                        self.auth_storage
                            .insert(config, &result.auth_backend, auth_data);
                        self.show_add_account = false;
                    }

                    self.auth_status = result.status;
                }

                BackgroundTaskResult::Cancelled => {
                    self.auth_status = AuthStatus::NotAuthorized;
                }
            }

            return true;
        }

        false
    }

    fn render_auth_window(&mut self, config: &mut Config, runtime: &Runtime, ui: &mut egui::Ui) {
        if let Some(message) = runtime.block_on(self.auth_message_provider.get_message()) {
            let lang = config.lang;
            let ctx = ui.ctx();

            egui::Window::new(LangMessage::Authorization.to_string(lang)).show(ctx, |ui| {
                ui.label(message.to_string(lang));
                let url = match message {
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
                ctx.include_bytes(uri, png_bytes.clone());
                ui.add(egui::Image::from_bytes(uri.to_string(), png_bytes));

                if ui.button(LangMessage::Cancel.to_string(lang)).clicked() {
                    self.auth_status = AuthStatus::NotAuthorized;
                    self.auth_task = None;
                    self.auth_message_provider = Arc::new(AuthMessageProvider::new(ctx));
                    self.on_instance_changed(config, runtime, ctx);
                }
            });
        }

        if runtime.block_on(self.auth_message_provider.need_offline_nickname()) {
            let lang = config.lang;
            let ctx = ui.ctx();

            let mut open = true;
            egui::Window::new(LangMessage::Authorization.to_string(lang))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(LangMessage::EnterNickname.to_string(lang));
                            ui.text_edit_singleline(&mut self.offline_nickname);
                        });

                        if ui
                            .add_enabled(
                                is_valid_minecraft_username(&self.offline_nickname),
                                egui::Button::new(LangMessage::AddAccount.to_string(lang)),
                            )
                            .clicked()
                        {
                            runtime.block_on(
                                self.auth_message_provider
                                    .set_offline_nickname(self.offline_nickname.clone()),
                            );
                        }
                    });
                });
            if !open {
                self.auth_status = AuthStatus::NotAuthorized;
                self.auth_task = None;
                self.auth_message_provider = Arc::new(AuthMessageProvider::new(ctx));
                self.on_instance_changed(config, runtime, ctx);
            }
        }
    }

    fn get_type_display_name(lang: Lang, new_account_type: NewAccountType) -> String {
        match new_account_type {
            NewAccountType::Microsoft => "Microsoft".to_string(),
            NewAccountType::ElyBy => "Ely.by".to_string(),
            NewAccountType::Telegram => "Telegram".to_string(),
            NewAccountType::Offline => LangMessage::Offline.to_string(lang),
        }
    }

    pub fn render_new_account_window(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        runtime: &Runtime,
        lang: Lang,
    ) {
        let mut show_add_account = self.show_add_account;
        Window::new(LangMessage::AddAccount.to_string(lang))
            .open(&mut show_add_account)
            .show(ui.ctx(), |ui| {
                ui.label(LangMessage::SelectAccount.to_string(lang));
                ComboBox::from_id_salt("new_account_type")
                    .selected_text(Self::get_type_display_name(lang, self.new_account_type))
                    .show_ui(ui, |ui| {
                        for account_type in [
                            NewAccountType::Microsoft,
                            NewAccountType::ElyBy,
                            NewAccountType::Telegram,
                            NewAccountType::Offline,
                        ] {
                            ui.selectable_value(
                                &mut self.new_account_type,
                                account_type,
                                Self::get_type_display_name(lang, account_type),
                            );
                        }
                    });

                match self.new_account_type {
                    NewAccountType::Microsoft => {}
                    NewAccountType::ElyBy => {
                        ui.horizontal(|ui| {
                            ui.label("Client ID:");
                            ui.text_edit_singleline(&mut self.ely_by_client_id);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Client Secret:");
                            ui.text_edit_singleline(&mut self.ely_by_client_secret);
                        });
                    }
                    NewAccountType::Telegram => {
                        ui.horizontal(|ui| {
                            ui.label("Auth Base URL:");
                            ui.text_edit_singleline(&mut self.telegram_auth_base_url);
                        });
                    }
                    NewAccountType::Offline => {}
                }

                if ui
                    .button(LangMessage::AddAndAuthenticate.to_string(lang))
                    .clicked()
                {
                    let new_auth_backend = match self.new_account_type {
                        NewAccountType::Microsoft => AuthBackend::Microsoft,
                        NewAccountType::ElyBy => AuthBackend::ElyBy(ElyByAuthBackend {
                            client_id: self.ely_by_client_id.clone(),
                            client_secret: self.ely_by_client_secret.clone(),
                        }),
                        NewAccountType::Telegram => AuthBackend::Telegram(TelegramAuthBackend {
                            auth_base_url: self.telegram_auth_base_url.clone(),
                        }),
                        NewAccountType::Offline => AuthBackend::Offline,
                    };

                    self.auth_status = AuthStatus::NotAuthorized;
                    self.auth_message_provider = Arc::new(AuthMessageProvider::new(ctx));
                    self.auth_task = Some(authenticate(
                        runtime,
                        None,
                        &new_auth_backend,
                        self.auth_message_provider.clone(),
                        ctx,
                    ));

                    self.show_add_account = false;
                }
            });
        self.show_add_account = show_add_account;
    }

    fn get_selected_storage_entry(&self, config: &Config) -> Option<StorageEntry> {
        let auth_profile = config.get_selected_auth_profile()?;
        self.auth_storage
            .get_by_id(&auth_profile.auth_backend_id, &auth_profile.username)
    }

    fn on_instance_changed(&mut self, config: &mut Config, runtime: &Runtime, ctx: &egui::Context) {
        self.auth_status = AuthStatus::NotAuthorized;

        let mut auth_profile = config.get_selected_auth_profile().cloned();

        if let Some(profile) = &auth_profile
            && self
                .auth_storage
                .get_by_id(&profile.auth_backend_id, &profile.username)
                .is_none()
        {
            config.clear_selected_auth_profile();
            auth_profile = None;
        }

        let storage_entry = self.get_selected_storage_entry(config);
        if let Some(storage_entry) = &storage_entry {
            if storage_entry.source == AuthDataSource::Persistent && self.auth_task.is_none() {
                self.auth_message_provider = Arc::new(AuthMessageProvider::new(ctx));
                self.auth_task = Some(authenticate(
                    runtime,
                    Some(storage_entry.auth_data.clone()),
                    &AuthBackend::from_id(&auth_profile.as_ref().unwrap().auth_backend_id),
                    self.auth_message_provider.clone(),
                    ctx,
                ));
            }
            if storage_entry.source == AuthDataSource::Runtime {
                self.auth_status = AuthStatus::Authorized;
            }
        }
    }

    fn render_buttons(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut Config,
        runtime: &Runtime,
        instance_auth_backend: Option<&AuthBackend>,
    ) {
        let mut auth_profile = config.get_selected_auth_profile().cloned();

        if ui
            .add_enabled(auth_profile.is_some(), egui::Button::new("-"))
            .clicked()
            && let Some(auth_profile) = auth_profile.take()
        {
            self.auth_storage.delete_by_id(
                config,
                &auth_profile.auth_backend_id,
                &auth_profile.username,
            );
            config.clear_selected_auth_profile();
        }

        if ui.button("+").clicked() {
            if let Some(instance_auth_backend) = instance_auth_backend {
                let ctx = ui.ctx();

                self.auth_status = AuthStatus::NotAuthorized;
                self.auth_message_provider = Arc::new(AuthMessageProvider::new(ctx));
                self.auth_task = Some(authenticate(
                    runtime,
                    None,
                    instance_auth_backend,
                    self.auth_message_provider.clone(),
                    ctx,
                ));
            } else {
                self.show_add_account = true;

                self.new_account_type = NewAccountType::Microsoft;

                self.ely_by_client_id = String::new();
                self.ely_by_client_secret = String::new();

                self.telegram_auth_base_url = String::new();

                self.offline_nickname = String::new();
            }
        }
    }

    fn get_account_display_name((id, username): &(String, String)) -> String {
        let backend = AuthBackend::from_id(id);
        let provider = get_auth_provider(&backend);
        let provider_name = provider.get_name();

        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        let hash = hasher.finish();

        let hex = format!("{hash:X}");
        format!("{} ({} #{})", username, provider_name, &hex[0..4])
    }

    fn get_combobox_text(
        nickname: &str,
        status: &AuthStatus,
        lang: Lang,
        dark_mode: bool,
    ) -> RichText {
        match status {
            AuthStatus::Authorized => RichText::new(nickname).color(colors::ok(dark_mode)),
            AuthStatus::NotAuthorized => RichText::new(format!(
                "{} ({})",
                nickname,
                LangMessage::Authorizing.to_string(lang)
            ))
            .color(colors::in_progress(dark_mode)),
            AuthStatus::AuthorizeError => RichText::new(format!(
                "{} ({})",
                nickname,
                LangMessage::UnknownAuthError.to_string(lang)
            ))
            .color(colors::error(dark_mode)),
            AuthStatus::AuthorizeErrorOffline => RichText::new(format!(
                "{} ({})",
                nickname,
                LangMessage::Offline.to_string(lang)
            ))
            .color(colors::offline(dark_mode)),
            AuthStatus::AuthorizeErrorTimeout => RichText::new(format!(
                "{} ({})",
                nickname,
                LangMessage::AuthTimeout.to_string(lang)
            ))
            .color(colors::timeout(dark_mode)),
        }
    }

    pub fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut Config,
        runtime: &Runtime,
        ctx: &egui::Context,
        instance_auth_backend: Option<&AuthBackend>,
    ) {
        let lang = config.lang;

        let auth_profile = config.get_selected_auth_profile().cloned();
        if auth_profile.is_none()
            && let Some(instance_auth_backend) = instance_auth_backend
        {
            let entries = self
                .auth_storage
                .get_id_nicknames(&instance_auth_backend.get_id());
            if let Some(nickname) = entries.first() {
                let auth_profile_value = AuthProfile {
                    auth_backend_id: instance_auth_backend.get_id(),
                    username: nickname.clone(),
                };
                config.set_selected_auth_profile(auth_profile_value.clone());
            }
        }

        if self.last_auth_profile != auth_profile {
            self.on_instance_changed(config, runtime, ctx);
            self.last_auth_profile = auth_profile.clone();
        }

        let dark_mode = ui.style().visuals.dark_mode;

        let auth_profile = config.get_selected_auth_profile().cloned();
        if let Some(instance_auth_backend) = instance_auth_backend {
            let mut entries = self
                .auth_storage
                .get_id_nicknames(&instance_auth_backend.get_id());

            if !entries.is_empty() {
                self.render_buttons(ui, config, runtime, Some(instance_auth_backend));

                let mut selected_username = auth_profile.as_ref().map(|x| x.username.to_string());
                ComboBox::from_id_salt("select_account")
                    .selected_text(match &selected_username {
                        Some(username) => {
                            Self::get_combobox_text(username, &self.auth_status, lang, dark_mode)
                        }
                        None => RichText::new(LangMessage::SelectAccount.to_string(lang))
                            .color(colors::action(dark_mode)),
                    })
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        if config.selected_instance_name.is_none() {
                            ui.disable();
                            return;
                        }
                        entries.sort();
                        for username in entries {
                            ui.selectable_value(
                                &mut selected_username,
                                Some(username.clone()),
                                username,
                            );
                        }
                    });
                if let Some(selected_username) = selected_username
                    && auth_profile.as_ref().map(|x| &x.username) != Some(&selected_username)
                {
                    let auth_profile_value = AuthProfile {
                        auth_backend_id: instance_auth_backend.get_id(),
                        username: selected_username.clone(),
                    };
                    config.set_selected_auth_profile(auth_profile_value.clone());
                }
            } else {
                ui.add_enabled_ui(self.auth_task.is_none(), |ui| {
                    let auth_provider = get_auth_provider(instance_auth_backend);
                    let button_text = if !matches!(instance_auth_backend, AuthBackend::Offline) {
                        egui::RichText::new(
                            LangMessage::AuthorizeUsing(auth_provider.get_name()).to_string(lang),
                        )
                    } else {
                        egui::RichText::new(LangMessage::AddOfflineAccount.to_string(lang))
                    }
                    .size(20.0);

                    if ui
                        .add_sized([ui.available_width(), 50.0], egui::Button::new(button_text))
                        .clicked()
                    {
                        let storage_entry = self.get_selected_storage_entry(config);

                        self.auth_status = AuthStatus::NotAuthorized;
                        self.auth_message_provider = Arc::new(AuthMessageProvider::new(ctx));
                        self.auth_task = Some(authenticate(
                            runtime,
                            storage_entry.as_ref().map(|x| x.auth_data.clone()),
                            instance_auth_backend,
                            self.auth_message_provider.clone(),
                            ctx,
                        ));
                    }
                });
            }
        } else {
            self.render_buttons(ui, config, runtime, instance_auth_backend);

            let mut all_entries = self.auth_storage.get_all_entries();

            let mut selected_account = auth_profile
                .as_ref()
                .map(|x| (x.auth_backend_id.clone(), x.username.clone()));
            ComboBox::from_id_salt("select_account")
                .selected_text(match &selected_account {
                    Some((_, username)) => {
                        Self::get_combobox_text(username, &self.auth_status, lang, dark_mode)
                    }
                    None => RichText::new(LangMessage::SelectAccount.to_string(lang))
                        .color(colors::action(dark_mode)),
                })
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    if config.selected_instance_name.is_none() {
                        ui.disable();
                        return;
                    }
                    all_entries.sort();
                    for (id, username) in all_entries {
                        ui.selectable_value(
                            &mut selected_account,
                            Some((id.clone(), username.clone())),
                            Self::get_account_display_name(&(id, username)),
                        );
                    }
                });
            if let Some(selected_account) = selected_account
                && auth_profile
                    .as_ref()
                    .map(|x| (&x.auth_backend_id, &x.username))
                    != Some((&selected_account.0, &selected_account.1))
            {
                let auth_profile_value = AuthProfile {
                    auth_backend_id: selected_account.0,
                    username: selected_account.1,
                };
                config.set_selected_auth_profile(auth_profile_value.clone());
            }
        }

        self.render_new_account_window(ui, ctx, runtime, lang);
        self.render_auth_window(config, runtime, ui);
    }

    pub fn get_auth_data(&self, config: &Config) -> Option<AuthData> {
        let profile = config.get_selected_auth_profile()?;
        if let Some(storage_entry) = self
            .auth_storage
            .get_by_id(&profile.auth_backend_id, &profile.username)
        {
            match storage_entry.source {
                AuthDataSource::Runtime => {
                    return Some(storage_entry.auth_data);
                }
                AuthDataSource::Persistent => {
                    if self.auth_status == AuthStatus::AuthorizeErrorOffline {
                        return Some(storage_entry.auth_data); // allow playing with an existing account when offline
                    }
                    return None;
                }
            }
        }
        None
    }

    pub fn offline(&self) -> bool {
        matches!(
            self.auth_status,
            AuthStatus::AuthorizeErrorOffline
                | AuthStatus::AuthorizeErrorTimeout
                | AuthStatus::AuthorizeError
        )
    }

    pub fn reset(&mut self, config: &mut Config, runtime: &Runtime, ctx: &egui::Context) {
        self.on_instance_changed(config, runtime, ctx);
    }
}
