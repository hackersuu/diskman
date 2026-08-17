# diskman

Config'de tanımlı harici diskleri oturum açılışında otomatik bağlayan bir Linux kullanıcı servisi. LUKS şifreli diskler keyfile veya `systemd-ask-password` ile desteklenir.

## Özellikler

- Sadece `config.toml`'da tanımlı diskler bağlanır — tanımsız diskler dokunulmaz
- LUKS şifre çözme: keyfile veya interaktif parola
- FS tipi `blkid` ile otomatik tespit edilir (ext4, btrfs, xfs, vfat, ntfs, exfat...)
- `systemd --user` oneshot servisi — oturum açılınca çalışır, işi bitince kapanır
- `--dry-run` ile test modunda çalıştırılabilir

## Gereksinimler

- `cryptsetup` (LUKS diskler için)
- `blkid` (util-linux paketi)
- `ntfs-3g` (isteğe bağlı, NTFS için)
- Rust toolchain (build için)

## Kurulum

```bash
# Build
cargo build --release

# Binary'yi PATH'e kopyala
sudo cp target/release/diskman /usr/local/bin/

# Config oluştur
mkdir -p ~/.config/diskman
cp config/diskman.toml.example ~/.config/diskman/config.toml
# Kendi UUID ve label'larınızla düzenleyin:
nano ~/.config/diskman/config.toml

# systemd --user servisi kur
mkdir -p ~/.config/systemd/user
cp systemd/diskman.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable diskman.service
```

## Kullanım

```bash
# Bir kere elle çalıştır
diskman

# Dry-run: neyin mount edileceğini göster, işlem yapma
diskman --dry-run

# Alternatif config ile çalıştır
diskman --config /path/to/config.toml

# Logları izle
journalctl --user -u diskman -f
```

## Config Formatı

Config dosyası: `~/.config/diskman/config.toml` (yoksa `/etc/diskman/config.toml`)

```toml
[general]
mount_base = "/mnt/diskman"
log_level  = "info"

# Normal disk
[[disk]]
uuid  = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
label = "Yedek"
mount_options = "defaults,noatime"

# Keyfile ile LUKS
[[disk]]
uuid      = "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy"
label     = "GizliDisk"
encrypted = true
keyfile   = "/etc/diskman/keys/gizlidisk.key"

# Parola sorulsun
[[disk]]
uuid      = "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz"
encrypted = true
```

## UUID Öğrenme

```bash
sudo blkid
# veya
lsblk -f
```
