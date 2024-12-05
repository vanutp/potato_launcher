use super::{
    elyby::ElyByAuthProvider, none::NoneAuthProvider, telegram::TGAuthProvider,
    version_auth_data::UserInfo,
};
use crate::auth::microsoft::MicrosoftAuthProvider;
use crate::message_provider::MessageProvider;
use async_trait::async_trait;
use shared::version::extra_version_metadata::AuthData;

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
    async fn authenticate(&self, message_provider: &dyn MessageProvider) -> anyhow::Result<AuthState>;

    async fn refresh(&self, refresh_token: String) -> anyhow::Result<AuthState>;

    async fn get_user_info(&self, token: &str) -> anyhow::Result<AuthState>;

    fn get_auth_url(&self) -> Option<String>;

    fn get_name(&self) -> String;
}

pub fn get_auth_provider(auth_data: &AuthData) -> Box<dyn AuthProvider + Send + Sync> {
    match auth_data {
        AuthData::Microsoft => Box::new(MicrosoftAuthProvider::new()),

        AuthData::ElyBy(auth_data) => Box::new(ElyByAuthProvider::new(
            &auth_data.client_id,
            &auth_data.client_secret,
        )),

        AuthData::Telegram(auth_data) => Box::new(TGAuthProvider::new(&auth_data.auth_base_url)),

        AuthData::None => Box::new(NoneAuthProvider::new()),
    }
}
