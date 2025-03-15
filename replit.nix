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
  ];
}