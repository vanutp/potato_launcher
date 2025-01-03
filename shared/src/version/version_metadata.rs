use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncReadExt as _};

use crate::{
    files::{self, CheckEntry},
    paths::get_metadata_path,
    progress,
};

use super::version_manifest::MetadataInfo;

#[derive(Deserialize, Serialize, Clone)]
pub struct Os {
    pub name: Option<String>,
    pub arch: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Rule {
    pub action: String,
    pub os: Option<Os>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ArgumentValue {
    String(String),
    Array(Vec<String>),
}

impl ArgumentValue {
    pub fn get_values(&self) -> Vec<&str> {
        match self {
            ArgumentValue::String(s) => vec![s.as_str()],
            ArgumentValue::Array(a) => a.iter().map(|x| x.as_str()).collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ComplexArgument {
    pub value: ArgumentValue,
    pub rules: Vec<Rule>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum VariableArgument {
    Simple(String),
    Complex(ComplexArgument),
}

impl VariableArgument {
    pub fn get_values(&self) -> Vec<&str> {
        match self {
            VariableArgument::Simple(s) => vec![s.as_str()],
            VariableArgument::Complex(c) => c.value.get_values(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Arguments {
    pub game: Vec<VariableArgument>,
    pub jvm: Vec<VariableArgument>,
}

#[derive(Deserialize, Serialize)]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub url: String,
}

#[derive(Deserialize, Serialize)]
pub struct JavaVersion {
    #[serde(rename = "majorVersion")]
    pub major_version: u64,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Download {
    pub sha1: String,
    pub url: String,
}

impl Download {
    pub fn get_check_download_entry(&self, path: &Path) -> CheckEntry {
        CheckEntry {
            url: self.url.clone(),
            remote_sha1: Some(self.sha1.clone()),
            path: path.to_path_buf(),
        }
    }

    pub fn get_filename(&self) -> &str {
        self.url.split('/').last().unwrap_or(&self.url)
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryDownloads {
    pub artifact: Option<Download>,
    pub classifiers: Option<HashMap<String, Download>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct LibraryExtract {
    pub exclude: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Library {
    name: String,
    pub downloads: Option<LibraryDownloads>,
    pub rules: Option<Vec<Rule>>,
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub natives: Option<HashMap<String, String>>,
    extract: Option<LibraryExtract>,
}

impl Library {
    pub fn from_download(name: String, url: String, sha1: String) -> Self {
        Library {
            name,
            downloads: Some(LibraryDownloads {
                artifact: Some(Download { url, sha1 }),
                classifiers: None,
            }),
            rules: None,
            url: None,
            sha1: None,
            natives: None,
            extract: None,
        }
    }

    pub fn get_path_from_name(&self) -> String {
        let full_name = self.name.clone();
        let mut parts: Vec<&str> = full_name.split(':').collect();
        if parts.len() != 4 {
            parts.push("");
        }
        let (pkg, name, version, suffix) = (parts[0], parts[1], parts[2], parts[3]);
        // neoforge adds "@jar" to the version, so we need to remove it
        let version = version.split("@jar").next().unwrap();
        let pkg_path = pkg.replace('.', "/");
        let suffix = if suffix.is_empty() {
            "".to_string()
        } else {
            format!("-{}", suffix)
        };
        format!(
            "{}/{}/{}/{}-{}{}.jar",
            pkg_path, name, version, name, version, suffix
        )
    }

    fn get_library_dir(&self, libraries_dir: &Path) -> PathBuf {
        let path = libraries_dir.join(self.get_path_from_name());
        path.parent().unwrap_or(libraries_dir).to_path_buf()
    }

    pub fn get_natives_path(
        &self,
        natives_name: &str,
        natives_download: &Download,
        libraries_dir: &Path,
    ) -> PathBuf {
        let path = self
            .get_library_dir(libraries_dir)
            .join(natives_name)
            .join(natives_download.get_filename());
        path
    }

    pub fn get_path(&self, libraries_dir: &Path) -> Option<PathBuf> {
        if let Some(downloads) = &self.downloads {
            if downloads.artifact.is_some() {
                return Some(libraries_dir.join(&self.get_path_from_name()));
            }
        }
        if self.url.is_some() {
            return Some(libraries_dir.join(&self.get_path_from_name()));
        }

        None
    }

    fn get_check_download_entry(&self, libraries_dir: &Path) -> Option<CheckEntry> {
        if let Some(url) = &self.url {
            return Some(CheckEntry {
                url: format!("{}/{}", url, self.get_path_from_name()),
                remote_sha1: self.sha1.clone(),
                path: libraries_dir.join(&self.get_path_from_name()),
            });
        }
        if let Some(downloads) = &self.downloads {
            if let Some(artifact) = &downloads.artifact {
                if let Some(path) = self.get_path(libraries_dir) {
                    return Some(artifact.get_check_download_entry(&path));
                }
            }
        }

        None
    }

    fn get_natives_check_download_entries(&self, libraries_dir: &Path) -> Vec<CheckEntry> {
        let mut entries = vec![];

        if let Some(downloads) = &self.downloads {
            if let Some(classifiers) = &downloads.classifiers {
                for (natives_name, download) in classifiers {
                    entries.push(CheckEntry {
                        url: download.url.clone(),
                        remote_sha1: Some(download.sha1.clone()),
                        path: libraries_dir.join(self.get_natives_path(
                            natives_name,
                            download,
                            libraries_dir,
                        )),
                    });
                }
            }
        }

        entries
    }

    pub fn get_all_check_download_entries(&self, libraries_dir: &Path) -> Vec<CheckEntry> {
        let mut entries = vec![];
        if let Some(entry) = self.get_check_download_entry(libraries_dir) {
            entries.push(entry);
        }
        entries.extend(self.get_natives_check_download_entries(libraries_dir));

        entries
    }

    pub fn get_natives_download(&self, natives_name: &str) -> Option<&Download> {
        let downloads = self.downloads.as_ref()?;
        let classifiers = downloads.classifiers.as_ref()?;
        let download = classifiers.get(natives_name)?;
        Some(download)
    }

    pub fn get_specific_check_download_entries(
        &self,
        natives_name: Option<&str>,
        libraries_dir: &Path,
    ) -> Vec<CheckEntry> {
        let mut entries = vec![];
        if let Some(natives_name) = natives_name {
            if let Some(download) = self.get_natives_download(natives_name) {
                let path = self.get_natives_path(natives_name, download, libraries_dir);
                entries.push(download.get_check_download_entry(&path));
            }
        }
        if let Some(entry) = self.get_check_download_entry(libraries_dir) {
            entries.push(entry);
        }

        entries
    }

    pub fn get_sha1_url(&self) -> Option<String> {
        Some(self.url.clone()? + &self.get_path_from_name() + ".sha1")
    }

    pub fn get_extract(&self) -> Option<&LibraryExtract> {
        self.extract.as_ref()
    }

    pub fn get_group_id(&self) -> String {
        let parts: Vec<&str> = self.name.split(':').collect();
        parts[0].to_string()
    }

    pub fn get_full_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_name_without_version(&self) -> String {
        let mut parts: Vec<&str> = self.name.split(':').collect();
        if parts.len() != 4 {
            parts.push("");
        }
        parts.remove(2);
        parts.join(":")
    }
}

#[derive(Deserialize, Serialize)]
pub struct Downloads {
    pub client: Option<Download>,
}

#[derive(Deserialize, Serialize)]
pub struct VersionMetadata {
    pub arguments: Option<Arguments>,

    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,

    pub downloads: Option<Downloads>,
    pub id: String,

    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,

    #[serde(rename = "mainClass")]
    pub main_class: String,

    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,

    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
}

lazy_static::lazy_static! {
    static ref LEGACY_JVM_ARGS: Vec<VariableArgument> = vec![
        VariableArgument::Complex(ComplexArgument {
            value: ArgumentValue::String("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump".to_string()),
            rules: vec![Rule{
                action: "allow".to_string(),
                os: Some(Os {
                    name: Some("windows".to_string()),
                    arch: None,
                }),
                features: None,
            }],
        }),
        VariableArgument::Complex(ComplexArgument {
            value: ArgumentValue::Array(vec!["-Dos.name=Windows 10".to_string(), "-Dos.version=10.0".to_string()]),
            rules: vec![Rule{
                action: "allow".to_string(),
                os: Some(Os {
                    name: Some("windows".to_string()),
                    arch: None,
                }),
                features: None,
            }],
        }),
        VariableArgument::Simple("-Djava.library.path=${natives_directory}".to_string()),
        VariableArgument::Simple("-Dminecraft.launcher.brand=${launcher_name}".to_string()),
        VariableArgument::Simple("-Dminecraft.launcher.version=${launcher_version}".to_string()),
        VariableArgument::Simple("-cp".to_string()),
        VariableArgument::Simple("${classpath}".to_string()),
    ];
}

impl VersionMetadata {
    pub async fn read_local(versions_dir: &Path, version_id: &str) -> anyhow::Result<Self> {
        let version_path = get_metadata_path(versions_dir, version_id);
        let mut file = fs::File::open(version_path).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        let metadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }

    pub async fn fetch(url: &str) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();
        let response = client.get(url).send().await?.error_for_status()?;
        let metadata = response.json().await?;
        Ok(metadata)
    }

    pub fn get_check_entry(metadata_info: &MetadataInfo, versions_dir: &Path) -> CheckEntry {
        let url = metadata_info.url.clone();
        let sha1 = metadata_info.sha1.clone();
        let path = get_metadata_path(versions_dir, &metadata_info.id);
        CheckEntry {
            url,
            remote_sha1: Some(sha1),
            path,
        }
    }

    pub async fn read_or_download(
        metadata_info: &MetadataInfo,
        versions_dir: &Path,
    ) -> anyhow::Result<Self> {
        let check_entry = Self::get_check_entry(metadata_info, versions_dir);
        let check_entries = vec![check_entry];
        let download_entries =
            files::get_download_entries(check_entries, progress::no_progress_bar()).await?;
        files::download_files(download_entries, progress::no_progress_bar()).await?;
        Self::read_local(versions_dir, &metadata_info.id).await
    }

    pub fn get_arguments(&self) -> anyhow::Result<Arguments> {
        match &self.arguments {
            Some(arguments) => Ok(arguments.clone()),
            None => {
                let minecraft_arguments = self.minecraft_arguments.clone().unwrap();
                Ok(Arguments {
                    game: minecraft_arguments
                        .split_whitespace()
                        .map(|x| VariableArgument::Simple(x.to_string()))
                        .collect(),
                    jvm: LEGACY_JVM_ARGS.clone(),
                })
            }
        }
    }

    pub async fn save(&self, versions_dir: &Path) -> anyhow::Result<()> {
        let version_path = get_metadata_path(versions_dir, &self.id);
        let content = serde_json::to_string(self)?;
        fs::write(version_path, content).await?;
        Ok(())
    }
}
