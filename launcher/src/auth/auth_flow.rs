use std::sync::Arc;

use tokio::sync::{Mutex, mpsc};

use crate::lang::LangMessage;

use super::base::{AuthProvider, AuthResultData, AuthState};
use super::user_info::AuthData;

struct AuthMessageState {
    auth_message: Option<LangMessage>,
    need_offline_nickname: u32,
}

pub struct AuthMessageProvider {
    state: Arc<Mutex<AuthMessageState>>,
    offline_nickname_sender: mpsc::UnboundedSender<String>,
    offline_nickname_receiver: Arc<Mutex<mpsc::UnboundedReceiver<String>>>,
    ctx: egui::Context,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Auth loop exceeded max iterations")]
    InfiniteAuthLoop,
}

impl AuthMessageProvider {
    pub fn new(ctx: &egui::Context) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            state: Arc::new(Mutex::new(AuthMessageState {
                auth_message: None,
                need_offline_nickname: 0,
            })),
            offline_nickname_sender: sender,
            offline_nickname_receiver: Arc::new(Mutex::new(receiver)),
            ctx: ctx.clone(),
        }
    }

    pub async fn set_message(&self, message: LangMessage) {
        if matches!(
            message,
            LangMessage::AuthMessage { .. } | LangMessage::DeviceAuthMessage { .. }
        ) {
            let mut state = self.state.lock().await;
            state.auth_message = Some(message);
            self.ctx.request_repaint();
        } else {
            panic!("Expected AuthMessage, got {message:?}");
        }
    }

    pub async fn get_message(&self) -> Option<LangMessage> {
        let state = self.state.lock().await;
        state.auth_message.clone()
    }

    pub async fn clear(&self) {
        let mut state = self.state.lock().await;
        state.auth_message = None;
        self.ctx.request_repaint();
    }

    pub async fn request_offline_nickname(&self) -> String {
        {
            let mut state = self.state.lock().await;
            state.need_offline_nickname += 1;
        }

        self.offline_nickname_receiver
            .lock()
            .await
            .recv()
            .await
            .unwrap()
    }

    pub async fn need_offline_nickname(&self) -> bool {
        let state = self.state.lock().await;
        state.need_offline_nickname > 0
    }

    pub async fn set_offline_nickname(&self, nickname: String) {
        let mut state = self.state.lock().await;
        state.need_offline_nickname -= 1;
        self.offline_nickname_sender.send(nickname).unwrap();
    }
}

pub async fn perform_auth(
    auth_data: Option<AuthData>,
    auth_provider: Box<dyn AuthProvider + Send + Sync>,
    auth_message_provider: Arc<AuthMessageProvider>,
) -> anyhow::Result<AuthData> {
    let mut auth_result_data = auth_data.map(|data| AuthResultData {
        access_token: data.access_token,
        refresh_token: data.refresh_token,
    });
    let mut auth_state = auth_result_data
        .clone()
        .map_or(AuthState::Auth, AuthState::UserInfo);

    for _ in 0..10 {
        match auth_state {
            AuthState::Auth => {
                auth_state = auth_provider
                    .authenticate(auth_message_provider.as_ref())
                    .await?;
            }

            AuthState::Refresh => {
                let refresh_token = auth_result_data
                    .as_ref()
                    .and_then(|data| data.refresh_token.clone());
                auth_state = match refresh_token {
                    Some(refresh_token) => auth_provider.refresh(refresh_token).await?,
                    None => AuthState::Auth,
                };
            }

            AuthState::UserInfo(data) => {
                auth_result_data = Some(data.clone());
                auth_state = auth_provider
                    .get_user_info(&data.access_token)
                    .await
                    .or_else(|e| {
                        let is_client_error = e
                            .downcast_ref::<reqwest::Error>()
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
                return Ok(AuthData {
                    access_token: auth_result_data.access_token,
                    refresh_token: auth_result_data.refresh_token,
                    user_info: info,
                });
            }
        }
    }

    Err(AuthError::InfiniteAuthLoop.into())
}
