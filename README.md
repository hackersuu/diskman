# diskman 🚀

[![License: GPL v2](https://img.shields.io/badge/License-GPL_v2-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)](https://kernel.org)

**diskman**, Linux ortamında yalnızca yapılandırma dosyasında (`config.toml`) tanımlanan harici diskleri oturum açılışında otomatik olarak tespit edip bağlayan (mount eden), şifreli (LUKS) diskleri açabilen modern ve hafif bir Rust kullanıcı servisidir (`systemd --user`).

---

## ✨ Özellikler

- 🔒 **Güvenli ve Kontrollü:** Yalnızca `config.toml` dosyasında açıkça tanımladığınız diskleri bağlar, tanımlanmamış diskleri görmezden gelir.
- 🔑 **LUKS / Şifreli Disk Desteği:**
  - Anahtar dosyası (**keyfile**) ile otomatik şifre çözme.
  - Keyfile belirtilmediğinde oturum açılışında `systemd-ask-password` aracılığıyla parola isteme.
- ⚡ **Otomatik Dosya Sistemi Tespiti:** `blkid` entegrasyonu sayesinde `ext4`, `btrfs`, `xfs`, `vfat`, `exfat`, `ntfs` vb. dosya sistemlerini otomatik tanır.
- 🛠️ **Hafif ve Oneshot:** Oturum başladığında çalışır, tüm tanımlı diskleri bağlayıp bellek harcamadan sonlanır (`Type=oneshot`).
- 🧪 **Dry-Run Modu:** `--dry-run` parametresiyle herhangi bir işlem yapmadan önce neyin nereye bağlanacağını simüle edebilme.

---

## 📦 Kurulum

### 1. Arch Linux / Pacman Paketi ile Kurulum (Önerilen)

```bash
git clone https://github.com/hackersuu/diskman.git
cd diskman

# Paketi derleyin ve sisteme kurun:
makepkg -si
```

### 2. Kaynak Koddan Manuel Kurulum

```bash
git clone https://github.com/hackersuu/diskman.git
cd diskman

# Release derlemesi
cargo build --release

# Binary'yi kopyalayın
sudo install -Dm755 target/release/diskman /usr/local/bin/diskman

# systemd servisini kullanıcı profiline ekleyin
mkdir -p ~/.config/systemd/user
cp systemd/diskman.service ~/.config/systemd/user/
```

---

## ⚙️ Yapılandırma (Config)

Örnek konfigürasyon dosyasını kullanıcı dizinine kopyalayın:

```bash
mkdir -p ~/.config/diskman
cp /usr/share/doc/diskman/diskman.toml.example ~/.config/diskman/config.toml
# veya proje dizininden:
# cp config/diskman.toml.example ~/.config/diskman/config.toml
```

### Disk Bilgilerini (UUID / LABEL) Öğrenme:

```bash
sudo blkid
# veya
lsblk -f
```

### Örnek `~/.config/diskman/config.toml`:

```toml
[general]
# Mount noktalarının oluşturulacağı ana dizin (Varsayılan: /mnt/diskman)
mount_base = "/mnt/diskman"

# Log seviyesi: trace | debug | info | warn | error
log_level = "info"

# -------------------------------------------------------------
# 1. Normal (Şifresiz) Disk
# -------------------------------------------------------------
[[disk]]
uuid = "12345678-1234-1234-1234-123456789abc"
label = "YedekDisk"
mount_options = "defaults,noatime"

# -------------------------------------------------------------
# 2. Şifreli (LUKS) Disk - Keyfile ile otomatik açma
# -------------------------------------------------------------
[[disk]]
uuid = "87654321-4321-4321-4321-cba987654321"
label = "GuvenliDepo"
encrypted = true
keyfile = "/etc/diskman/keys/guvenli.key"
mount_options = "defaults,noatime"

# -------------------------------------------------------------
# 3. Şifreli (LUKS) Disk - Oturum açılışında parola sorma
# (keyfile verilmezse sistem parola penceresi / prompt açar)
# -------------------------------------------------------------
[[disk]]
uuid = "abcdefab-cdef-abcd-efab-cdefabcdefab"
label = "KisiselKasa"
encrypted = true
mount_options = "defaults,noatime,uid=1000,gid=1000"
```

---

## 🚀 Kullanım & Servisi Etkinleştirme

```bash
# Yapılandırmayı test etmek için (Hiçbir şeyi mount etmez):
diskman --dry-run

# Servisi systemd'ye tanıtın ve oturum açılışında çalışacak şekilde aktif edin:
systemctl --user daemon-reload
systemctl --user enable --now diskman.service

# Servis çıktılarını ve loglarını canlı izleyin:
journalctl --user -u diskman -f
```

---

## 📜 Komut Satırı Seçenekleri

```text
Kullanım: diskman [SEÇENEKLER]

Seçenekler:
  -c, --config <DOSYA>    Özel bir config dosyası yolu belirtir
  -n, --dry-run           Diskleri mount etmeden simülasyon yapar
  -h, --help              Yardım mesajını görüntüler
  -V, --version           Sürüm bilgisini görüntüler
```

---

## 📄 Lisans

Bu proje **GNU General Public License v2.0 (GPLv2)** altında lisanslanmıştır. Detaylar için [LICENSE](LICENSE) dosyasına göz atabilirsiniz.
