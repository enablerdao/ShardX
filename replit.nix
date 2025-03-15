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
    pkgs.rocksdb
    pkgs.cmake
    pkgs.gnumake
    pkgs.gcc
    pkgs.zlib
    pkgs.bzip2
    pkgs.lz4
    pkgs.snappy
    pkgs.zstd
  ];
  env = {
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    ROCKSDB_LIB_DIR = "${pkgs.rocksdb}/lib";
    ROCKSDB_STATIC = "1";
  };
}