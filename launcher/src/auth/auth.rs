use std::sync::Arc;
use std::sync::Mutex;

use shared::utils::BoxResult;

use crate::config::runtime_config::VersionAuthData;
use crate::lang::LangMessage;
use crate::message_provider::MessageProvider;

use super::base::{AuthProvider, AuthResultData, AuthState};

struct AuthMessageState {
    auth_message: Option<LangMessage>,
}

pub struct AuthMessageProvider {
    state: Arc<Mutex<AuthMessageState>>,
    ctx: egui::Context,
}

impl AuthMessageProvider {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            state: Arc::new(Mutex::new(AuthMessageState { auth_message: None })),
            ctx: ctx.clone(),
        }
    }
}


#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Auth loop exceeded max iterations")]
    InfiniteAuthLoop,
}


impl MessageProvider for AuthMessageProvider {
    fn set_message(&self, message: LangMessage) {
        if matches!(message, LangMessage::AuthMessage { .. } | LangMessage::DeviceAuthMessage { .. }) {
            let mut state = self.state.lock().unwrap();
            state.auth_message = Some(message);
            self.ctx.request_repaint();
        } else {
            panic!("Expected AuthMessage, got {:?}", message);
        }
    }

    fn get_message(&self) -> Option<LangMessage> {
        let state = self.state.lock().unwrap();
        return state.auth_message.clone();
    }

    fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.auth_message = None;
        self.ctx.request_repaint();
    }
}

pub async fn auth(
    auth_data: Option<VersionAuthData>,
    auth_provider: Arc<dyn AuthProvider + Send + Sync>,
    auth_message_provider: Arc<AuthMessageProvider>,
) -> BoxResult<VersionAuthData> {
    let mut auth_result_data = auth_data
        .map_or(None, |data| Some(AuthResultData {
            access_token: data.access_token,
            refresh_token: data.refresh_token,
        }));
    let mut auth_state = auth_result_data
        .clone()
        .map_or(AuthState::Auth, |data| AuthState::UserInfo(data));

    for _ in 0..10 {
        match auth_state {
            AuthState::Auth => {
                auth_state = auth_provider
                    .authenticate(auth_message_provider.clone())
                    .await?;
            }

            AuthState::Refresh => {
                let refresh_token = auth_result_data
                    .as_ref()
                    .and_then(|data| data.refresh_token.clone());
                auth_state = match refresh_token {
                    Some(refresh_token) => auth_provider.refresh(refresh_token).await?,
                    None => AuthState::Auth
                };
            }

            AuthState::UserInfo(data) => {
                auth_result_data = Some(data.clone());
                auth_state = auth_provider
                    .get_user_info(&data.access_token)
                    .await
                    .or_else(|e| {
                        let is_client_error = e.downcast_ref::<reqwest::Error>()
                            .and_then(|re| re.status())
                            .map(|status| status.is_client_error())
                            .unwrap_or(false);
                        if is_client_error {
                            Ok(AuthState::Refresh)
                        } else {
                            Err(e)
                        }
                    })?;
            }

            AuthState::Success(info) => {
                let auth_result_data = auth_result_data.unwrap();
                return Ok(VersionAuthData {
                    access_token: auth_result_data.access_token,
                    refresh_token: auth_result_data.refresh_token,
                    user_info: info,
                });
            }
        }
    }

    Err(Box::new(AuthError::InfiniteAuthLoop))
}
