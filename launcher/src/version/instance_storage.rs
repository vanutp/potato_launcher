use std::{collections::HashSet, path::Path};

use log::{error, warn};
use serde::{Deserialize, Serialize};
use shared::{
    paths::{get_instance_dir, get_local_instances_path},
    version::version_manifest::{VersionInfo, VersionManifest},
};
use tokio::task;

use crate::{config::runtime_config::Config, utils::get_temp_dir};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum InstanceStatus {
    Missing,
    Outdated,
    UpToDate,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LocalInstance {
    pub version_info: VersionInfo,
    pub status: InstanceStatus,
}

pub struct InstanceStorage {
    instances: Vec<LocalInstance>,
    remote_manifest: Option<VersionManifest>,
}

impl InstanceStorage {
    pub async fn load(config: &Config) -> InstanceStorage {
        let local_instances_path = get_local_instances_path(&config.get_launcher_dir());
        let instances = match tokio::fs::read(local_instances_path).await {
            Ok(data) => {
                let instances: Vec<LocalInstance> =
                    serde_json::from_slice(&data).unwrap_or(Vec::new());
                instances
            }
            Err(_) => Vec::new(),
        };

        InstanceStorage {
            instances,
            remote_manifest: None,
        }
    }

    pub async fn safe_save(&self, config: &Config) {
        let local_instances_path = get_local_instances_path(&config.get_launcher_dir());
        let data = serde_json::to_vec(&self.instances).unwrap();
        tokio::fs::write(local_instances_path, data).await.unwrap();
    }

    pub fn set_remote_manifest(&mut self, manifest: Option<VersionManifest>) {
        self.remote_manifest = manifest;
    }

    fn get_remote_versions(&self) -> Vec<&VersionInfo> {
        if let Some(remote_manifest) = &self.remote_manifest {
            remote_manifest.versions.iter().collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_all_names(&self) -> (Vec<String>, Vec<String>) {
        let local_names: HashSet<String> = self
            .instances
            .iter()
            .map(|x| x.version_info.get_name())
            .collect();
        let remote_names: Vec<String> = self
            .get_remote_versions()
            .iter()
            .map(|x| x.get_name())
            .filter(|x| !local_names.contains(x))
            .collect();

        let mut local_names: Vec<String> = local_names.into_iter().collect();
        local_names.sort();

        (local_names, remote_names)
    }

    pub async fn add_instance(&mut self, config: &Config, version_info: VersionInfo) {
        self.instances.push(LocalInstance {
            version_info,
            status: InstanceStatus::Outdated,
        });
        self.safe_save(config).await;
    }

    pub fn get_instance(&self, version_name: &str) -> Option<LocalInstance> {
        let local_instance = self
            .instances
            .iter()
            .find(|instance| instance.version_info.get_name() == version_name)
            .cloned();
        let remote_instance = self
            .get_remote_versions()
            .into_iter()
            .find(|x| x.get_name() == version_name)
            .map(|version_info| LocalInstance {
                version_info: version_info.clone(),
                status: InstanceStatus::Missing,
            });

        if let Some(mut remote_instance) = remote_instance {
            if let Some(instance) = local_instance {
                if remote_instance.version_info != instance.version_info {
                    remote_instance.status = InstanceStatus::Outdated;
                } else {
                    remote_instance.status = InstanceStatus::UpToDate;
                }
            }
            Some(remote_instance)
        } else {
            local_instance
        }
    }

    pub async fn mark_downloaded(&mut self, config: &Config, version_name: &str) {
        let remote_versions = self.get_remote_versions();
        let remote_version = remote_versions
            .into_iter()
            .find(|v| v.get_name() == version_name)
            .cloned();

        if let Some(remote_version) = remote_version {
            self.instances
                .retain(|instance| instance.version_info.get_name() != version_name);
            self.instances.push(LocalInstance {
                version_info: remote_version,
                status: InstanceStatus::UpToDate,
            });
            self.safe_save(config).await;
        } else if let Some(instance) = self
            .instances
            .iter_mut()
            .find(|instance| instance.version_info.get_name() == version_name)
        {
            instance.status = InstanceStatus::UpToDate;
            self.safe_save(config).await;
        } else {
            warn!("Tried to mark non-existent version as downloaded: {version_name}");
        }
    }

    async fn remove_instance_files(&self, launcher_dir: &Path, version_name: &str) {
        let instance_dir = get_instance_dir(launcher_dir, version_name);
        if instance_dir.exists() {
            let unique_temp_dir;
            let mut i = 0;
            loop {
                let temp_dir = get_temp_dir().join(format!("{version_name}_{i}"));
                if !temp_dir.exists() {
                    unique_temp_dir = temp_dir;
                    break;
                }
                i += 1;
            }

            if let Err(e) = tokio::fs::rename(&instance_dir, &unique_temp_dir).await {
                error!("Error moving instance directory:\n{e:?}");
            } else {
                task::spawn(async move {
                    if let Err(e) = tokio::fs::remove_dir_all(&unique_temp_dir).await {
                        error!("Error deleting temporary directory:\n{e:?}");
                    }
                });
            }
        }
    }

    pub async fn delete_instance(&mut self, config: &Config, version_name: &str) {
        let launcher_dir = config.get_launcher_dir();

        let instance = self
            .instances
            .iter()
            .find(|instance| instance.version_info.get_name() == version_name);
        if let Some(instance) = instance {
            if instance.status != InstanceStatus::Outdated {
                self.remove_instance_files(&launcher_dir, version_name)
                    .await;
            }
            self.instances
                .retain(|instance| instance.version_info.get_name() != version_name);
            self.safe_save(config).await;
        }
    }
}
