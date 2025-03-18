{ pkgs }: {
  deps = [
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
    pkgs.rust-analyzer
    pkgs.pkg-config
    pkgs.openssl
    pkgs.openssl.dev
    pkgs.libiconv
    pkgs.nodePackages.npm
    pkgs.nodejs
    pkgs.clang
    pkgs.llvmPackages.libclang
    pkgs.llvmPackages.libcxxClang
    pkgs.rocksdb
    pkgs.cmake
    pkgs.gnumake
    pkgs.gcc
    pkgs.zlib
    pkgs.bzip2
    pkgs.lz4
    pkgs.snappy
    pkgs.zstd
    pkgs.netcat
    pkgs.curl
  ];
  env = {
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
    ROCKSDB_STATIC = "1";
    OPENSSL_DIR = "${pkgs.openssl.dev}";
    OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
    OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    RUSTFLAGS = "-C link-arg=-fuse-ld=lld";
  };
}