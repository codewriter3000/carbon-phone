cargo clean
PKG_CONFIG_ALLOW_CROSS=1 \
PKG_CONFIG_DIR= \
PKG_CONFIG_SYSROOT_DIR=/ \
PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/pkgconfig \
cargo build --release --target aarch64-unknown-linux-gnu

scp target/aarch64-unknown-linux-gnu/release/sparky testbench@192.168.1.22:~/sparky

