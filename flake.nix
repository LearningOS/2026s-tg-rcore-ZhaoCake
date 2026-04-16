{
  description = "tg-rcore-tutorial dev shell with pinned qemu 7.2.1";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixpkgs-qemu7.url = "github:NixOS/nixpkgs/986a0a1a1905b3227b28db009619321105c62464";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, nixpkgs-qemu7, flake-utils, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        pkgsQemu7 = import nixpkgs-qemu7 { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustup
            git
            gnumake
            pkg-config
            ripgrep
            which
          ] ++ [
            pkgsQemu7.qemu
          ];

          shellHook = ''
            echo "[tg-rcore-tutorial] Nix dev shell loaded"
            echo "QEMU pinned from nixpkgs rev: 986a0a1a1905b3227b28db009619321105c62464"
            qemu-system-riscv64 --version | head -n 1 || true
            echo "Rust toolchain is managed by rustup (per chapter rust-toolchain.toml)."
          '';
        };
      }
    );
}
