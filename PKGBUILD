pkgname=goat
pkgver=0.0.2
pkgrel=1
pkgdesc="goat - better sleep"
arch=('x86_64')
url="https://github.com/brocode/goat"
license=('WTFPL')
makedepends=('go')
source=('https://github.com/brocode/goat/releases/download/0.0.2/goat-linux-amd64')
sha256sums=('42b7f02a3210bb1dc3bbf5bf65d4222714d82e16ba451cae5de3608e7dc5c60c')

package() {
  mkdir -p "${pkgdir}/usr/bin"
  cp goat-linux-amd64 "${pkgdir}/usr/bin/goat"
}

