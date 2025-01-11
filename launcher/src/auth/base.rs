use super::auth_flow::AuthMessageProvider;
use super::offline::OfflineAuthProvider;
use super::{elyby::ElyByAuthProvider, telegram::TGAuthProvider, user_info::UserInfo};
use crate::auth::microsoft::MicrosoftAuthProvider;
use async_trait::async_trait;
use shared::version::extra_version_metadata::AuthBackend;

#[derive(Clone)]
pub struct AuthResultData {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub enum AuthState {
    Auth,
    Refresh,
    UserInfo(AuthResultData),
    Success(UserInfo),
}

#[async_trait]
pub trait AuthProvider {
    async fn authenticate(
        &self,
        message_provider: &AuthMessageProvider,
    ) -> anyhow::Result<AuthState>;

    async fn refresh(&self, refresh_token: String) -> anyhow::Result<AuthState>;

    async fn get_user_info(&self, token: &str) -> anyhow::Result<AuthState>;

    fn get_auth_url(&self) -> Option<String>;

    fn get_name(&self) -> String;
}

pub fn get_auth_provider(auth_backend: &AuthBackend) -> Box<dyn AuthProvider + Send + Sync> {
    match auth_backend {
        AuthBackend::Microsoft => Box::new(MicrosoftAuthProvider::new()),

        AuthBackend::ElyBy(auth_data) => Box::new(ElyByAuthProvider::new(
            &auth_data.client_id,
            &auth_data.client_secret,
        )),

        AuthBackend::Telegram(auth_data) => Box::new(TGAuthProvider::new(&auth_data.auth_base_url)),

        AuthBackend::Offline => Box::new(OfflineAuthProvider::new()),
    }
}
