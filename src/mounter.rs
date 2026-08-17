/// Mount/umount modülü.
/// Blok cihazı veya mapper cihazını belirtilen dizine bağlar.
use std::path::PathBuf;
use std::process::Command;
use crate::error::{DiskmanError, Result};

/// Cihazı verilen mount noktasına bağla.
/// Mount noktası yoksa otomatik oluşturulur.
/// `fs_type` None ise kernel otomatik tespit eder.
pub fn mount(
    device_path: &str,
    mountpoint: &PathBuf,
    options: &str,
    fs_type: Option<&str>,
) -> Result<()> {
    // Mount noktası dizinini oluştur (sudo ile)
    if !mountpoint.exists() {
        let status = Command::new("sudo")
            .args(["mkdir", "-p", mountpoint.to_str().unwrap_or("")])
            .status()
            .map_err(|e| DiskmanError::Mount(format!("mkdir çalıştırılamadı: {e}")))?;

        if !status.success() {
            return Err(DiskmanError::Mount(format!(
                "Mount noktası oluşturulamadı: {}",
                mountpoint.display()
            )));
        }
        tracing::debug!("Mount noktası oluşturuldu: {}", mountpoint.display());
    }

    let mut cmd = Command::new("sudo");
    cmd.arg("mount");

    // Dosya sistemi tipi belirtildiyse -t ekle
    if let Some(fs) = fs_type {
        // ntfs için ntfs-3g kullan
        let fs_driver = if fs == "ntfs" { "ntfs-3g" } else { fs };
        cmd.args(["-t", fs_driver]);
    }

    // Mount seçenekleri
    if !options.is_empty() {
        cmd.args(["-o", options]);
    }

    cmd.arg(device_path);
    cmd.arg(mountpoint.to_str().unwrap_or(""));

    tracing::info!(
        "Mount ediliyor: {} → {}",
        device_path,
        mountpoint.display()
    );

    let output = cmd.output().map_err(|e| {
        DiskmanError::Mount(format!("mount komutu çalıştırılamadı: {e}"))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(DiskmanError::Mount(format!(
            "mount başarısız: {} → {}\n  Hata: {stderr}",
            device_path,
            mountpoint.display()
        )));
    }

    tracing::info!(
        "✓ Mount edildi: {} → {}",
        device_path,
        mountpoint.display()
    );
    Ok(())
}

/// Verilen mount noktasını bağlantıdan çıkar.
#[allow(dead_code)]
pub fn umount(mountpoint: &PathBuf) -> Result<()> {
    let mp_str = mountpoint.to_str().unwrap_or("");

    tracing::info!("Umount yapılıyor: {}", mp_str);

    let status = Command::new("sudo")
        .args(["umount", mp_str])
        .status()
        .map_err(|e| DiskmanError::Mount(format!("umount çalıştırılamadı: {e}")))?;

    if !status.success() {
        return Err(DiskmanError::Mount(format!(
            "umount başarısız: {mp_str}"
        )));
    }

    // Boş dizini temizle
    if mountpoint.exists() {
        let _ = std::fs::remove_dir(mountpoint);
    }

    tracing::info!("Umount tamamlandı: {mp_str}");
    Ok(())
}

/// Config'deki disk adına göre mount noktası yolu üret.
/// Örnek: /mnt/diskman/MyLabel  veya  /mnt/diskman/uuid-xxxx
pub fn make_mountpoint(
    base: &PathBuf,
    label: Option<&str>,
    uuid: Option<&str>,
) -> PathBuf {
    let name = label
        .or(uuid)
        .unwrap_or("diskman_disk");

    // Güvenli dizin adı: boşluk ve özel karakterleri alt çizgiyle değiştir
    let safe_name: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect();

    base.join(safe_name)
}
