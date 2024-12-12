use crate::message_provider::MessageProvider;

use super::{
    base::{AuthProvider, AuthResultData, AuthState},
    user_info::UserInfo,
};
use async_trait::async_trait;
use uuid::Uuid;

pub struct OfflineAuthProvider {
    nickname: String,
}

impl OfflineAuthProvider {
    pub fn new(nickname: &str) -> Self {
        OfflineAuthProvider {
            nickname: nickname.to_string(),
        }
    }
}

#[async_trait]
impl AuthProvider for OfflineAuthProvider {
    async fn authenticate(&self, _: &dyn MessageProvider) -> anyhow::Result<AuthState> {
        Ok(AuthState::UserInfo(AuthResultData {
            access_token: "".to_string(),
            refresh_token: None,
        }))
    }

    async fn refresh(&self, _: String) -> anyhow::Result<AuthState> {
        Ok(AuthState::Auth)
    }

    async fn get_user_info(&self, _: &str) -> anyhow::Result<AuthState> {
        let namespace = Uuid::NAMESPACE_DNS;
        let name = format!("{}", self.nickname);
        let generated_uuid = Uuid::new_v3(&namespace, name.as_bytes());

        Ok(AuthState::Success(UserInfo {
            uuid: generated_uuid.to_string(),
            username: self.nickname.clone(),
        }))
    }

    fn get_auth_url(&self) -> Option<String> {
        None
    }

    fn get_name(&self) -> String {
        "Offline".to_string()
    }
}
