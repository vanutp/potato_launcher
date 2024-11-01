use super::{elyby::ElyByAuthProvider, none::NoneAuthProvider, telegram::TGAuthProvider};
use crate::auth::microsoft::MicrosoftAuthProvider;
use crate::message_provider::MessageProvider;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shared::{utils::BoxResult, version::extra_version_metadata::AuthData};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct UserInfo {
    pub uuid: String,
    pub username: String,
}

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
    async fn authenticate(&self, message_provider: Arc<dyn MessageProvider>) -> BoxResult<AuthState>;

    async fn refresh(&self, refresh_token: String) -> BoxResult<AuthState>;

    async fn get_user_info(&self, token: &str) -> BoxResult<AuthState>;

    fn get_auth_url(&self) -> Option<String>;

    fn get_name(&self) -> String;
}

pub fn get_auth_provider(auth_data: &AuthData) -> Arc<dyn AuthProvider + Send + Sync> {
    match auth_data {
        AuthData::Microsoft => Arc::new(MicrosoftAuthProvider::new()),

        AuthData::ElyBy(auth_data) => Arc::new(ElyByAuthProvider::new(
            &auth_data.client_id,
            &auth_data.client_secret,
        )),

        AuthData::Telegram(auth_data) => Arc::new(TGAuthProvider::new(&auth_data.auth_base_url)),

        AuthData::None => Arc::new(NoneAuthProvider::new()),
    }
}
