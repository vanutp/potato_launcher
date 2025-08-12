use std::collections::{HashMap, HashSet};

use shared::{paths::get_auth_data_path, version::extra_version_metadata::AuthBackend};

use crate::config::runtime_config::Config;

use super::user_info::AuthData;

#[derive(PartialEq)]
pub enum AuthDataSource {
    Persistent,
    Runtime,
}

pub struct StorageEntry {
    pub auth_data: AuthData,
    pub source: AuthDataSource,
}

pub struct AuthStorage {
    // saved auth data from previous launches, may not be up to date
    persistent_storage: HashMap<String, HashMap<String, AuthData>>, // backend id -> username -> auth data
    // auth data from current launch, up to date
    runtime_storage: HashMap<String, HashMap<String, AuthData>>, // backend id -> username -> auth data
}

impl AuthStorage {
    pub fn load(config: &Config) -> Self {
        let launcher_dir = config.get_launcher_dir();
        let auth_data_path = get_auth_data_path(&launcher_dir);
        let persistent_storage = match std::fs::read_to_string(&auth_data_path) {
            Ok(data) => {
                let auth_data: HashMap<String, HashMap<String, AuthData>> =
                    serde_json::from_str(&data).unwrap_or(HashMap::new());
                auth_data
            }
            Err(_) => HashMap::new(),
        };

        Self {
            persistent_storage,
            runtime_storage: HashMap::new(),
        }
    }

    pub fn get_by_id(&self, id: &str, username: &str) -> Option<StorageEntry> {
        if let Some(user_map) = self.runtime_storage.get(id)
            && let Some(auth_data) = user_map.get(username)
        {
            Some(StorageEntry {
                auth_data: auth_data.clone(),
                source: AuthDataSource::Runtime,
            })
        } else if let Some(user_map) = self.persistent_storage.get(id)
            && let Some(auth_data) = user_map.get(username)
        {
            Some(StorageEntry {
                auth_data: auth_data.clone(),
                source: AuthDataSource::Persistent,
            })
        } else {
            None
        }
    }

    pub fn get_id_nicknames(&self, id: &str) -> Vec<String> {
        let mut nicknames = HashSet::new();
        if let Some(user_map) = self.runtime_storage.get(id) {
            nicknames.extend(user_map.keys().cloned());
        }
        if let Some(user_map) = self.persistent_storage.get(id) {
            nicknames.extend(user_map.keys().cloned());
        }
        nicknames.into_iter().collect()
    }

    fn save(&self, config: &Config) {
        let launcher_dir = config.get_launcher_dir();
        let auth_data_path = get_auth_data_path(&launcher_dir);
        if let Ok(auth_data_str) = serde_json::to_string(&self.persistent_storage) {
            let _ = std::fs::write(&auth_data_path, auth_data_str);
        }
    }

    pub fn insert(&mut self, config: &Config, backend: &AuthBackend, auth_data: AuthData) {
        let id = backend.get_id();
        let username = auth_data.user_info.username.clone();

        self.runtime_storage
            .entry(id.clone())
            .or_default()
            .insert(username.clone(), auth_data.clone());
        self.persistent_storage
            .entry(id)
            .or_default()
            .insert(username, auth_data);

        self.save(config);
    }

    pub fn delete_by_id(&mut self, config: &Config, id: &str, username: &str) {
        if let Some(user_map) = self.runtime_storage.get_mut(id) {
            user_map.remove(username);
            if user_map.is_empty() {
                self.runtime_storage.remove(id);
            }
        }
        if let Some(user_map) = self.persistent_storage.get_mut(id) {
            user_map.remove(username);
            if user_map.is_empty() {
                self.persistent_storage.remove(id);
            }
        }

        self.save(config);
    }

    pub fn get_all_entries(&self) -> Vec<(String, String)> {
        let mut entries = HashMap::new();

        let mut collect_entries = |storage: &HashMap<String, HashMap<String, AuthData>>| {
            for (id, user_map) in storage {
                for username in user_map.keys() {
                    entries.insert(id.clone(), username.clone());
                }
            }
        };

        collect_entries(&self.persistent_storage);
        collect_entries(&self.runtime_storage);

        let mut result: Vec<_> = entries.into_iter().collect();
        result.sort();
        result
    }
}
