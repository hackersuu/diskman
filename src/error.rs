use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiskmanError {
    #[error("Config hatası: {0}")]
    Config(String),

    #[error("Disk tarama hatası: {0}")]
    Scanner(String),

    #[error("LUKS hatası: {0}")]
    Luks(String),

    #[error("Mount hatası: {0}")]
    Mount(String),

    #[error("IO hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("Komut başarısız: {command} → {stderr}")]
    #[allow(dead_code)]
    CommandFailed { command: String, stderr: String },
}

pub type Result<T> = std::result::Result<T, DiskmanError>;
