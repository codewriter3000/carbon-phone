cargo clean
PKG_CONFIG_ALLOW_CROSS=1 \
PKG_CONFIG_DIR= \
PKG_CONFIG_SYSROOT_DIR=/ \
PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig \
cargo build --release --target aarch64-unknown-linux-gnu -vv

