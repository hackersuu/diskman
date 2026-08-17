/// LUKS şifre çözme modülü.
/// cryptsetup luksOpen ile şifreli hacimleri açar.
/// Önce keyfile dener, yoksa systemd-ask-password kullanır.
use std::path::PathBuf;
use std::process::Command;
use crate::error::{DiskmanError, Result};

/// LUKS volume'ü aç ve mapped device adını döndür.
///
/// # Dönüş değeri
/// `/dev/mapper/<mapper_name>` — mount için kullanılacak cihaz yolu.
pub fn open_luks(
    device_path: &str,
    mapper_name: &str,
    keyfile: Option<&PathBuf>,
) -> Result<String> {
    let mapper_path = format!("/dev/mapper/{mapper_name}");

    // Zaten açık mı?
    if std::path::Path::new(&mapper_path).exists() {
        tracing::info!("LUKS zaten açık: {mapper_path}");
        return Ok(mapper_path);
    }

    if let Some(kf) = keyfile {
        tracing::info!("LUKS açılıyor (keyfile): {} → {}", device_path, mapper_name);
        open_with_keyfile(device_path, mapper_name, kf)?;
    } else {
        tracing::info!("LUKS açılıyor (şifre sorulacak): {} → {}", device_path, mapper_name);
        open_with_password(device_path, mapper_name)?;
    }

    Ok(mapper_path)
}

/// cryptsetup luksOpen ile keyfile kullanarak aç.
fn open_with_keyfile(device_path: &str, mapper_name: &str, keyfile: &PathBuf) -> Result<()> {
    if !keyfile.exists() {
        return Err(DiskmanError::Luks(format!(
            "Keyfile bulunamadı: {}",
            keyfile.display()
        )));
    }

    let status = Command::new("cryptsetup")
        .args([
            "luksOpen",
            "--batch-mode",
            "--key-file",
            keyfile.to_str().unwrap_or(""),
            device_path,
            mapper_name,
        ])
        .status()
        .map_err(|e| DiskmanError::Luks(format!("cryptsetup çalıştırılamadı: {e}")))?;

    if !status.success() {
        return Err(DiskmanError::Luks(format!(
            "cryptsetup luksOpen başarısız: {} (keyfile: {})",
            device_path,
            keyfile.display()
        )));
    }

    tracing::info!("LUKS açıldı (keyfile): /dev/mapper/{mapper_name}");
    Ok(())
}

/// systemd-ask-password ile kullanıcıdan parola alarak aç.
fn open_with_password(device_path: &str, mapper_name: &str) -> Result<()> {
    // systemd-ask-password ile parolayı al
    let ask_output = Command::new("systemd-ask-password")
        .arg(format!(
            "'{mapper_name}' diski için LUKS parolası girin:"
        ))
        .output()
        .map_err(|e| {
            DiskmanError::Luks(format!("systemd-ask-password çalıştırılamadı: {e}"))
        })?;

    if !ask_output.status.success() {
        return Err(DiskmanError::Luks(
            "Parola alınamadı (systemd-ask-password başarısız)".to_string(),
        ));
    }

    let password = String::from_utf8_lossy(&ask_output.stdout);
    let password = password.trim_end_matches('\n');

    // Parolayı stdin üzerinden cryptsetup'a ver
    let mut child = Command::new("cryptsetup")
        .args(["luksOpen", "--batch-mode", device_path, mapper_name])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| DiskmanError::Luks(format!("cryptsetup spawn hatası: {e}")))?;

    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin
            .write_all(password.as_bytes())
            .map_err(|e| DiskmanError::Luks(format!("stdin yazma hatası: {e}")))?;
    }

    let status = child
        .wait()
        .map_err(|e| DiskmanError::Luks(format!("cryptsetup bekleme hatası: {e}")))?;

    if !status.success() {
        return Err(DiskmanError::Luks(format!(
            "cryptsetup luksOpen başarısız (yanlış parola?): {}",
            device_path
        )));
    }

    tracing::info!("LUKS açıldı (parola ile): /dev/mapper/{mapper_name}");
    Ok(())
}

/// LUKS volume'ü kapat (mount edilmişse önce umount yapılmalı).
pub fn close_luks(mapper_name: &str) -> Result<()> {
    let mapper_path = format!("/dev/mapper/{mapper_name}");

    if !std::path::Path::new(&mapper_path).exists() {
        tracing::debug!("LUKS zaten kapalı: {mapper_name}");
        return Ok(());
    }

    let status = Command::new("cryptsetup")
        .args(["luksClose", mapper_name])
        .status()
        .map_err(|e| DiskmanError::Luks(format!("cryptsetup luksClose çalıştırılamadı: {e}")))?;

    if !status.success() {
        return Err(DiskmanError::Luks(format!(
            "cryptsetup luksClose başarısız: {mapper_name}"
        )));
    }

    tracing::info!("LUKS kapatıldı: {mapper_name}");
    Ok(())
}

/// UUID veya label'dan mapper ismi üret (güvenli, dosya sistemi uyumlu).
pub fn make_mapper_name(uuid: Option<&str>, label: Option<&str>) -> String {
    let base = label
        .or(uuid)
        .unwrap_or("diskman_luks");

    // Sadece alfanumerik + tire + alt çizgi bırak
    base.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
