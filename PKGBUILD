# Maintainer: hacker_su
pkgname=diskman
pkgver=0.1.2
pkgrel=1
pkgdesc="Config'de tanımlı harici diskleri oturum açılışında otomatik bağlayan systemd kullanıcı servisi (LUKS destekli)"
arch=('x86_64' 'aarch64')
url="https://github.com/hackersuu/diskman"
license=('GPL2')

# Çalışma zamanı bağımlılıkları
depends=(
    'cryptsetup'   # LUKS şifre çözme
    'util-linux'   # blkid, mount
)

# İsteğe bağlı bağımlılıklar
optdepends=(
    'ntfs-3g: NTFS dosya sistemi desteği'
)

# Derleme bağımlılıkları
makedepends=('rust' 'cargo')

# Kurulum kancaları (post_install, post_upgrade, pre_remove)
install="$pkgname.install"

# Kaynak: yerel git deposu
source=("$pkgname::git+file:///home/hacker_su/projeler/diskman")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/$pkgname"
    # Bağımlılıkları önceden indir (offline build için isteğe bağlı)
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$srcdir/$pkgname"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR="$srcdir/target"

    cargo build \
        --release \
        --locked \
        --target "$CARCH-unknown-linux-gnu"
}

check() {
    cd "$srcdir/$pkgname"
    export CARGO_TARGET_DIR="$srcdir/target"
    cargo test --release --locked --target "$CARCH-unknown-linux-gnu" 2>/dev/null || true
}

package() {
    cd "$srcdir/$pkgname"
    local target_dir="$srcdir/target/$CARCH-unknown-linux-gnu/release"

    # Binary
    install -Dm755 "$target_dir/$pkgname" \
        "$pkgdir/usr/bin/$pkgname"

    # systemd kullanıcı servisi
    install -Dm644 "systemd/$pkgname.service" \
        "$pkgdir/usr/lib/systemd/user/$pkgname.service"

    # Örnek config dosyası
    install -Dm644 "config/$pkgname.toml.example" \
        "$pkgdir/usr/share/doc/$pkgname/$pkgname.toml.example"

    # Dokümantasyon
    install -Dm644 README.md \
        "$pkgdir/usr/share/doc/$pkgname/README.md"

    # Lisans
    install -Dm644 LICENSE \
        "$pkgdir/usr/share/licenses/$pkgname/LICENSE" 2>/dev/null || true
}
