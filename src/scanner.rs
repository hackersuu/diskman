/// Disk tarama modülü.
/// Config'deki her disk girişi için blkid ile cihazı bulur.
use std::process::Command;
use crate::config::DiskConfig;
use crate::error::{DiskmanError, Result};

/// blkid'den dönen disk bilgisi
#[derive(Debug, Clone)]
pub struct BlockDevice {
    /// /dev/sdX veya /dev/nvmeXn1pX gibi cihaz yolu
    pub path: String,
    /// Dosya sistemi tipi (ext4, btrfs, crypto_LUKS, vfat, ...)
    pub fs_type: Option<String>,
    /// Disk etiketi
    pub label: Option<String>,
    /// UUID
    pub uuid: Option<String>,
}

impl BlockDevice {
    /// Bu cihaz bir LUKS şifreli volume mu?
    pub fn is_luks(&self) -> bool {
        self.fs_type
            .as_deref()
            .map(|t| t == "crypto_LUKS")
            .unwrap_or(false)
    }
}

/// Config'deki disk girdisine karşılık gelen blok cihazını bul.
/// UUID veya LABEL ile `blkid` kullanarak arar.
pub fn find_device(disk: &DiskConfig) -> Result<Option<BlockDevice>> {
    // UUID ile ara (öncelikli)
    if let Some(ref uuid) = disk.uuid {
        if let Some(dev) = blkid_by_uuid(uuid)? {
            return Ok(Some(dev));
        }
    }

    // LABEL ile ara
    if let Some(ref label) = disk.label {
        if let Some(dev) = blkid_by_label(label)? {
            return Ok(Some(dev));
        }
    }

    Ok(None)
}

/// `blkid -U <uuid>` ile cihaz yolunu bul, ardından detayları al.
fn blkid_by_uuid(uuid: &str) -> Result<Option<BlockDevice>> {
    let output = Command::new("blkid")
        .arg("-U")
        .arg(uuid)
        .output()
        .map_err(|e| DiskmanError::Scanner(format!("blkid çalıştırılamadı: {e}")))?;

    if !output.status.success() {
        // Bulunamadı → None
        return Ok(None);
    }

    let device_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if device_path.is_empty() {
        return Ok(None);
    }

    blkid_details(&device_path).map(Some)
}

/// `blkid -L <label>` ile cihaz yolunu bul, ardından detayları al.
fn blkid_by_label(label: &str) -> Result<Option<BlockDevice>> {
    let output = Command::new("blkid")
        .arg("-L")
        .arg(label)
        .output()
        .map_err(|e| DiskmanError::Scanner(format!("blkid çalıştırılamadı: {e}")))?;

    if !output.status.success() {
        return Ok(None);
    }

    let device_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if device_path.is_empty() {
        return Ok(None);
    }

    blkid_details(&device_path).map(Some)
}

/// `blkid -o export <device>` ile tam cihaz bilgisini al.
fn blkid_details(device_path: &str) -> Result<BlockDevice> {
    let output = Command::new("blkid")
        .args(["-o", "export", device_path])
        .output()
        .map_err(|e| DiskmanError::Scanner(format!("blkid -o export çalıştırılamadı: {e}")))?;

    let text = String::from_utf8_lossy(&output.stdout);
    let mut fs_type = None;
    let mut label = None;
    let mut uuid = None;

    for line in text.lines() {
        if let Some((key, val)) = line.split_once('=') {
            match key {
                "TYPE"  => fs_type = Some(val.to_string()),
                "LABEL" => label   = Some(val.to_string()),
                "UUID"  => uuid    = Some(val.to_string()),
                _ => {}
            }
        }
    }

    Ok(BlockDevice {
        path: device_path.to_string(),
        fs_type,
        label,
        uuid,
    })
}

/// Bir cihazın /proc/mounts içinde zaten mount edilip edilmediğini kontrol et.
pub fn is_mounted(device_path: &str) -> Result<bool> {
    let mounts = std::fs::read_to_string("/proc/mounts")
        .map_err(|e| DiskmanError::Scanner(format!("/proc/mounts okunamadı: {e}")))?;

    Ok(mounts.lines().any(|line| {
        line.split_whitespace()
            .next()
            .map(|dev| dev == device_path)
            .unwrap_or(false)
    }))
}

/// Bir mount noktasının /proc/mounts içinde kullanımda olup olmadığını kontrol et.
#[allow(dead_code)]
pub fn is_mountpoint_used(mountpoint: &str) -> Result<bool> {
    let mounts = std::fs::read_to_string("/proc/mounts")
        .map_err(|e| DiskmanError::Scanner(format!("/proc/mounts okunamadı: {e}")))?;

    Ok(mounts.lines().any(|line| {
        let mut parts = line.split_whitespace();
        parts.next(); // cihaz
        parts.next()  // mount noktası
            .map(|mp| mp == mountpoint)
            .unwrap_or(false)
    }))
}
