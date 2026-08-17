use serde::Deserialize;
use std::path::PathBuf;
use crate::error::{DiskmanError, Result};

/// Ana config yapısı
/// Dosya yolu: ~/.config/diskman/config.toml  veya  /etc/diskman/config.toml
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,

    /// Bağlanacak diskler listesi — sadece buradakiler işlenir
    #[serde(default)]
    pub disk: Vec<DiskConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    /// Mount noktalarının oluşturulacağı taban dizin
    #[serde(default = "default_mount_base")]
    pub mount_base: PathBuf,

    /// Log seviyesi: trace, debug, info, warn, error
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            mount_base: default_mount_base(),
            log_level: default_log_level(),
        }
    }
}

fn default_mount_base() -> PathBuf {
    PathBuf::from("/mnt/diskman")
}

fn default_log_level() -> String {
    "info".to_string()
}

/// Config'de tanımlı tek bir disk girişi.
/// UUID veya label ile tanımlanır — biri zorunlu.
#[derive(Debug, Deserialize, Clone)]
pub struct DiskConfig {
    /// Diskin blkid UUID'si
    pub uuid: Option<String>,

    /// Diskin etiketi (LABEL)
    pub label: Option<String>,

    /// Bu disk için mount seçenekleri
    #[serde(default = "default_mount_options")]
    pub mount_options: String,

    /// Şifreli (LUKS) disk mi?
    #[serde(default)]
    pub encrypted: bool,

    /// LUKS anahtar dosyası yolu.
    /// Belirtilmezse runtime'da systemd-ask-password ile parola sorulur.
    pub keyfile: Option<PathBuf>,
}

impl DiskConfig {
    /// Bu disk girişini tanımlamak için kullanılabilecek bir ad döner.
    pub fn display_name(&self) -> String {
        self.label
            .clone()
            .or_else(|| self.uuid.clone())
            .unwrap_or_else(|| "<adsız>".to_string())
    }
}

fn default_mount_options() -> String {
    "defaults,noatime".to_string()
}

impl Config {
    /// Config dosyasını yükle.
    /// Sırasıyla: override_path → ~/.config/diskman/config.toml → /etc/diskman/config.toml
    /// Hiçbiri bulunamazsa hata döner (config olmadan çalışmanın anlamı yok).
    pub fn load(override_path: Option<&PathBuf>) -> Result<Self> {
        let candidates: Vec<PathBuf> = if let Some(p) = override_path {
            vec![p.clone()]
        } else {
            let mut paths = Vec::new();
            if let Ok(home) = std::env::var("HOME") {
                paths.push(PathBuf::from(home).join(".config/diskman/config.toml"));
            }
            paths.push(PathBuf::from("/etc/diskman/config.toml"));
            paths
        };

        for path in &candidates {
            if path.exists() {
                let content = std::fs::read_to_string(path).map_err(|e| {
                    DiskmanError::Config(format!("'{}' okunamadı: {}", path.display(), e))
                })?;
                let config: Config = toml::from_str(&content).map_err(|e| {
                    DiskmanError::Config(format!("'{}' parse hatası: {}", path.display(), e))
                })?;
                tracing::info!("Config yüklendi: {}", path.display());

                if config.disk.is_empty() {
                    tracing::warn!("Config'de hiç [[disk]] girişi yok — yapılacak iş yok.");
                }

                // Her disk girişinin en az uuid veya label içermesini doğrula
                for (i, d) in config.disk.iter().enumerate() {
                    if d.uuid.is_none() && d.label.is_none() {
                        return Err(DiskmanError::Config(format!(
                            "disk[{}]: uuid veya label alanlarından biri zorunludur.",
                            i
                        )));
                    }
                }

                return Ok(config);
            }
        }

        Err(DiskmanError::Config(format!(
            "Config dosyası bulunamadı. Beklenen konumlar:\n  {}",
            candidates
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n  ")
        )))
    }
}
