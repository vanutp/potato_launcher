use async_trait::async_trait;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use reqwest::Client;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::time::sleep;

use crate::config::build_config;
use crate::lang::LangMessage;

use super::auth_flow::AuthMessageProvider;
use super::base::{AuthProvider, AuthResultData, AuthState};
use super::user_info::UserInfo;

const ELY_BY_BASE: &str = "https://ely.by/";

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid code")]
    InvalidCode,
    #[error("Invalid token type")]
    InvalidTokenType,
    #[error("Missing access token")]
    MissingAccessToken,
    #[error("Request error")]
    RequestError,
    #[error("Timeout during authentication")]
    AuthTimeout,
    #[error("Missing query string")]
    MissingQueryString,
}

pub struct ElyByAuthProvider {
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize)]
struct AuthQuery {
    code: String,
}

async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<String> {
    let client = Client::new();
    let resp = client
        .post("https://account.ely.by/api/oauth2/v1/token")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
            ("code", code),
        ])
        .send()
        .await?;

    let status = resp.status();
    let data: serde_json::Value = resp.json().await?;
    if status != 200 && data.get("error") == Some(&"invalid_request".into()) {
        return Err(AuthError::InvalidCode.into());
    }

    if data.get("token_type") != Some(&"Bearer".into()) {
        return Err(AuthError::InvalidTokenType.into());
    }

    if let Some(access_token) = data.get("access_token")
        && let Some(access_token) = access_token.as_str()
    {
        Ok(access_token.to_string())
    } else {
        Err(AuthError::MissingAccessToken.into())
    }
}

enum TokenResult {
    Token(String),
    InvalidCode,
    Error(anyhow::Error),
}

async fn handle_request(
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    req: Request<hyper::body::Incoming>,
    token_tx: Box<mpsc::UnboundedSender<TokenResult>>,
) -> anyhow::Result<Response<Full<Bytes>>> {
    let query = req.uri().query().ok_or(AuthError::MissingQueryString)?;
    let auth_query: AuthQuery = serde_urlencoded::from_str(query)?;

    let token_result =
        match exchange_code(&client_id, &client_secret, &auth_query.code, &redirect_uri).await {
            Ok(token) => TokenResult::Token(token),
            Err(e) => match e.downcast::<AuthError>() {
                Ok(e) => match e {
                    AuthError::InvalidCode => TokenResult::InvalidCode,
                    _ => TokenResult::Error(e.into()),
                },
                Err(e) => TokenResult::Error(e),
            },
        };

    let response = match &token_result {
        TokenResult::Token(_) => Response::builder()
            .status(302)
            .header(
                "Location",
                format!(
                    "https://account.ely.by/oauth2/code/success?appName={}",
                    &build_config::get_launcher_name(),
                ),
            )
            .body(Full::new(Bytes::from("")))?,

        TokenResult::InvalidCode => Response::builder()
            .status(400)
            .body(Full::new(Bytes::from("Invalid code")))?,

        TokenResult::Error(_) => Response::builder()
            .status(500)
            .body(Full::new(Bytes::from("Internal server error")))?,
    };

    let _ = token_tx.send(token_result);

    Ok(response)
}

impl ElyByAuthProvider {
    pub fn new(elyby_client_id: &str, elyby_client_secret: &str) -> Self {
        ElyByAuthProvider {
            client_id: elyby_client_id.to_string(),
            client_secret: elyby_client_secret.to_string(),
        }
    }

    async fn print_auth_url(&self, redirect_uri: &str, message_provider: &AuthMessageProvider) {
        let url = format!(
            "https://account.ely.by/oauth2/v1?client_id={}&redirect_uri={}&response_type=code&scope=account_info%20minecraft_server_session&prompt=select_account",
            &self.client_id, redirect_uri
        );
        let _ = open::that(&url);
        message_provider
            .set_message(LangMessage::AuthMessage { url })
            .await;
    }
}

#[async_trait]
impl AuthProvider for ElyByAuthProvider {
    async fn authenticate(
        &self,
        message_provider: &AuthMessageProvider,
    ) -> anyhow::Result<AuthState> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = TcpListener::bind(addr).await?;

        let redirect_uri = format!("http://localhost:{}/", listener.local_addr()?.port());
        self.print_auth_url(&redirect_uri, message_provider).await;

        let mut http = http1::Builder::new();
        http.keep_alive(false);

        loop {
            let stream;
            tokio::select! {
                _ = sleep(Duration::from_secs(120)) => {
                    return Err(AuthError::AuthTimeout.into());
                }

                st = listener.accept() => {
                    stream = st?.0;
                }
            }

            let io = TokioIo::new(stream);

            let (token_tx, mut token_rx) = mpsc::unbounded_channel();
            let token_tx = Box::new(token_tx);

            http.serve_connection(
                io,
                service_fn(|req: Request<hyper::body::Incoming>| {
                    let token_tx = token_tx.clone();
                    handle_request(
                        self.client_id.clone(),
                        self.client_secret.clone(),
                        redirect_uri.clone(),
                        req,
                        token_tx,
                    )
                }),
            )
            .await?;

            if let Some(token) = token_rx.recv().await {
                match token {
                    TokenResult::Token(token) => {
                        return Ok(AuthState::UserInfo(AuthResultData {
                            access_token: token,
                            refresh_token: None,
                        }));
                    }
                    TokenResult::InvalidCode => continue,
                    TokenResult::Error(e) => return Err(e),
                }
            } else {
                return Err(AuthError::RequestError.into());
            }
        }
    }

    async fn refresh(&self, _: String) -> anyhow::Result<AuthState> {
        Ok(AuthState::Auth)
    }

    async fn get_user_info(&self, token: &str) -> anyhow::Result<AuthState> {
        let client = Client::new();
        let resp: UserInfo = client
            .get("https://account.ely.by/api/account/v1/info")
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(AuthState::Success(resp))
    }

    fn get_auth_url(&self) -> Option<String> {
        Some(ELY_BY_BASE.to_string())
    }

    fn get_name(&self) -> String {
        "Ely.by".to_string()
    }
}
