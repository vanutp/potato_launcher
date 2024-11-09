use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use shared::{utils::BoxResult, version::extra_version_metadata::AuthData};

use crate::utils::get_data_dir;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct UserInfo {
    pub uuid: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct VersionAuthData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user_info: UserInfo,
}

#[derive(Serialize, Deserialize)]
pub struct AuthStorage {
    auth_data: HashMap<String, VersionAuthData>,
}

const AUTH_DATA_FILENAME: &str = "auth_data.json";

pub fn get_auth_data_path() -> PathBuf {
    get_data_dir().join(AUTH_DATA_FILENAME)
}

impl AuthStorage {
    pub fn load() -> Self {
        let auth_data_path = get_auth_data_path();
        if auth_data_path.exists() {
            let auth_data_str =
                std::fs::read_to_string(&auth_data_path).expect("Failed to read auth data file");
            if let Ok(auth_data) = serde_json::from_str(&auth_data_str) {
                return auth_data;
            }
        }

        let auth_data = AuthStorage {
            auth_data: HashMap::new(),
        };
        return auth_data;
    }

    pub fn get(&self, data: &AuthData) -> Option<&VersionAuthData> {
        self.auth_data.get(&data.get_id())
    }

    fn save(&self) -> BoxResult<()> {
        let auth_data_path = get_auth_data_path();
        let auth_data_str = serde_json::to_string(self).expect("Failed to serialize auth data");
        std::fs::write(&auth_data_path, auth_data_str)?;
        Ok(())
    }

    pub fn insert(&mut self, data: &AuthData, auth_data: VersionAuthData) -> BoxResult<()> {
        self.auth_data.insert(data.get_id(), auth_data);
        self.save()?;
        Ok(())
    }
}

pub struct RuntimeAuthStorage {
    auth_data: HashMap<String, VersionAuthData>,
}

impl RuntimeAuthStorage {
    pub fn new() -> Self {
        Self {
            auth_data: HashMap::new(),
        }
    }

    pub fn get(&self, data: &AuthData) -> Option<&VersionAuthData> {
        self.auth_data.get(&data.get_id())
    }

    pub fn insert(&mut self, data: &AuthData, auth_data: VersionAuthData) {
        self.auth_data.insert(data.get_id(), auth_data);
    }

    pub fn contains(&self, data: &AuthData) -> bool {
        self.auth_data.contains_key(&data.get_id())
    }
}
