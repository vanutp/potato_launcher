use crate::message_provider::MessageProvider;

use super::{
    base::{AuthProvider, AuthResultData, AuthState},
    version_auth_data::UserInfo,
};
use async_trait::async_trait;
use shared::utils::BoxResult;

pub struct NoneAuthProvider {}

impl NoneAuthProvider {
    pub fn new() -> Self {
        NoneAuthProvider {}
    }
}

#[async_trait]
impl AuthProvider for NoneAuthProvider {
    async fn authenticate(&self, _: &dyn MessageProvider) -> BoxResult<AuthState> {
        Ok(AuthState::UserInfo(AuthResultData {
            access_token: "".to_string(),
            refresh_token: None,
        }))
    }

    async fn refresh(&self, _: String) -> BoxResult<AuthState> {
        Ok(AuthState::Auth)
    }

    async fn get_user_info(&self, _: &str) -> BoxResult<AuthState> {
        Ok(AuthState::Success(UserInfo {
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            username: "demo".to_string(),
        }))
    }

    fn get_auth_url(&self) -> Option<String> {
        None
    }

    fn get_name(&self) -> String {
        "No auth provider".to_string()
    }
}
