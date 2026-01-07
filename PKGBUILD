# Maintainer: Kena <kena@kena.gay>
pkgname='applier'
pkgver=0.1.0
pkgrel=1
pkgdesc="Automatically emails employers"
arch=('x86_64')
license=('MIT')
depends=()
makedepends=('rust' 'cargo')
# source=("$pkgname-latest.tar.gz")
# sha256sums=('SKIP')

build() {
	# cd "$pkgname"
	# cargo build -r
	echo "hello"
}

package() {
	# cd "$pkgname"
	sudo install -Dm755 "target/release/$pkgname" "$pkdir/usr/bin/applier"
}
