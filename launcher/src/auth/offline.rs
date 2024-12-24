use super::{
    auth::AuthMessageProvider,
    base::{AuthProvider, AuthResultData, AuthState},
    user_info::UserInfo,
};
use async_trait::async_trait;
use uuid::Uuid;

pub struct OfflineAuthProvider {}

impl OfflineAuthProvider {
    pub fn new() -> Self {
        OfflineAuthProvider {}
    }
}

#[async_trait]
impl AuthProvider for OfflineAuthProvider {
    async fn authenticate(
        &self,
        message_provider: &AuthMessageProvider,
    ) -> anyhow::Result<AuthState> {
        Ok(AuthState::UserInfo(AuthResultData {
            access_token: message_provider.request_offline_nickname().await,
            refresh_token: None,
        }))
    }

    async fn refresh(&self, _: String) -> anyhow::Result<AuthState> {
        Ok(AuthState::Auth)
    }

    async fn get_user_info(&self, token: &str) -> anyhow::Result<AuthState> {
        let nickname = token;
        let namespace = Uuid::NAMESPACE_DNS;
        let generated_uuid = Uuid::new_v3(&namespace, nickname.as_bytes());

        Ok(AuthState::Success(UserInfo {
            uuid: generated_uuid.to_string(),
            username: nickname.to_string(),
        }))
    }

    fn get_auth_url(&self) -> Option<String> {
        None
    }

    fn get_name(&self) -> String {
        "Offline".to_string()
    }
}
