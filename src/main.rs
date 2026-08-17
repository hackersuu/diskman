mod config;
mod error;
mod luks;
mod mounter;
mod scanner;

use clap::Parser;
use std::path::PathBuf;
use tracing::{error, info, warn};

use config::Config;

#[derive(Parser, Debug)]
#[command(
    name = "diskman",
    version,
    about = "Config'de tanımlı diskleri otomatik olarak bağlar. LUKS şifreli diskler de desteklenir."
)]
struct Cli {
    /// Alternatif config dosyası yolu
    #[arg(short, long, value_name = "DOSYA")]
    config: Option<PathBuf>,

    /// Bağlamak yerine sadece disk durumlarını listele
    #[arg(short = 'n', long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Config yükle (tracing henüz kurulmadı, eprintln kullan)
    let cfg = match Config::load(cli.config.as_ref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[diskman] HATA: Config yüklenemedi — {e}");
            std::process::exit(1);
        }
    };

    // Loglama kur (config'den log seviyesi, RUST_LOG ile geçersiz kılınabilir)
    let log_level = cfg.general.log_level.clone();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level)),
        )
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    info!(
        "diskman başlatıldı — {} disk girişi işlenecek",
        cfg.disk.len()
    );

    if cfg.disk.is_empty() {
        warn!("Config'de hiç [[disk]] girişi yok. Çıkılıyor.");
        std::process::exit(0);
    }

    if cli.dry_run {
        info!("--- DRY RUN MODU (hiçbir şey mount edilmeyecek) ---");
    }

    let mount_base = &cfg.general.mount_base;
    let mut success = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    for disk in &cfg.disk {
        let disk_name = disk.display_name();
        info!("── İşleniyor: {disk_name}");

        // 1) Cihazı bul (UUID veya label ile)
        let device = match scanner::find_device(disk) {
            Ok(Some(d)) => d,
            Ok(None) => {
                warn!("  [{disk_name}] Disk bulunamadı (takılı değil veya UUID/label yanlış) — atlanıyor.");
                skipped += 1;
                continue;
            }
            Err(e) => {
                error!("  [{disk_name}] Tarama hatası: {e}");
                failed += 1;
                continue;
            }
        };

        info!(
            "  [{disk_name}] Cihaz: {} (fs: {})",
            device.path,
            device.fs_type.as_deref().unwrap_or("bilinmiyor")
        );

        // 2) LUKS mu? Gerekirse aç
        let mount_device = if disk.encrypted || device.is_luks() {
            let mapper_name = luks::make_mapper_name(
                device.uuid.as_deref(),
                device.label.as_deref(),
            );

            if cli.dry_run {
                info!("  [{disk_name}] [DRY] LUKS açılacak → /dev/mapper/{mapper_name}");
                skipped += 1;
                continue;
            }

            match luks::open_luks(&device.path, &mapper_name, disk.keyfile.as_ref()) {
                Ok(mapper_path) => mapper_path,
                Err(e) => {
                    error!("  [{disk_name}] LUKS açılamadı: {e}");
                    failed += 1;
                    continue;
                }
            }
        } else {
            device.path.clone()
        };

        // 3) Zaten mount edilmiş mi?
        match scanner::is_mounted(&mount_device) {
            Ok(true) => {
                info!("  [{disk_name}] Zaten mount edilmiş — atlanıyor.");
                skipped += 1;
                continue;
            }
            Err(e) => {
                error!("  [{disk_name}] Mount durumu kontrol edilemedi: {e}");
                failed += 1;
                continue;
            }
            Ok(false) => {}
        }

        // 4) Mount noktasını belirle: /mnt/diskman/<label|uuid>
        let mountpoint = mounter::make_mountpoint(
            mount_base,
            device.label.as_deref(),
            device.uuid.as_deref(),
        );

        if cli.dry_run {
            info!(
                "  [{disk_name}] [DRY] Mount edilecek: {} → {}",
                mount_device,
                mountpoint.display()
            );
            skipped += 1;
            continue;
        }

        // 5) Mount et
        let fs_type = device.fs_type.as_deref();
        match mounter::mount(&mount_device, &mountpoint, &disk.mount_options, fs_type) {
            Ok(()) => {
                info!(
                    "  [{disk_name}] ✓ Başarıyla bağlandı: {}",
                    mountpoint.display()
                );
                success += 1;
            }
            Err(e) => {
                error!("  [{disk_name}] ✗ Mount başarısız: {e}");
                // Mount başarısızsa LUKS'u tekrar kapat
                if disk.encrypted || device.is_luks() {
                    let mapper_name = luks::make_mapper_name(
                        device.uuid.as_deref(),
                        device.label.as_deref(),
                    );
                    let _ = luks::close_luks(&mapper_name);
                }
                failed += 1;
            }
        }
    }

    // Özet log
    info!(
        "── Tamamlandı: {} bağlandı, {} atlandı, {} başarısız",
        success, skipped, failed
    );

    if failed > 0 {
        std::process::exit(1);
    }
}
