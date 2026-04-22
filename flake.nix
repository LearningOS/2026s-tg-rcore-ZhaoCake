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
            bash
            cargo-clone
            cargo-binutils
            git
            gnumake
            pkg-config
            ripgrep
            which
          ] ++ [
            pkgsQemu7.qemu
          ];

          shellHook = ''
            # Force cargo/rustc/rustdoc to go through rustup stable toolchain
            # so chapter-local rust-toolchain targets resolve consistently.
            RUSTUP_BIN_DIR="$PWD/.direnv/rustup-bin"
            mkdir -p "$RUSTUP_BIN_DIR"

            cat > "$RUSTUP_BIN_DIR/cargo" <<'EOF'
#!/usr/bin/env bash
exec rustup run stable cargo "$@"
EOF

            cat > "$RUSTUP_BIN_DIR/rustc" <<'EOF'
#!/usr/bin/env bash
exec rustup run stable rustc "$@"
EOF

            cat > "$RUSTUP_BIN_DIR/rustdoc" <<'EOF'
#!/usr/bin/env bash
exec rustup run stable rustdoc "$@"
EOF

            chmod +x "$RUSTUP_BIN_DIR/cargo" "$RUSTUP_BIN_DIR/rustc" "$RUSTUP_BIN_DIR/rustdoc"
            export PATH="$RUSTUP_BIN_DIR:$PATH"

            TARGET_TRIPLE="riscv64gc-unknown-none-elf"
            TOOLCHAIN="stable-x86_64-unknown-linux-gnu"
            if ! rustup target list --installed --toolchain "$TOOLCHAIN" | grep -q "^$TARGET_TRIPLE$"; then
              echo "[tg-rcore-tutorial] installing missing rust target: $TARGET_TRIPLE"
              rustup target add "$TARGET_TRIPLE" --toolchain "$TOOLCHAIN"
            fi

            echo "[tg-rcore-tutorial] Nix dev shell loaded"
            echo "QEMU pinned from nixpkgs rev: 986a0a1a1905b3227b28db009619321105c62464"
            qemu-system-riscv64 --version | head -n 1 || true
            echo "Rust toolchain is managed by rustup (per chapter rust-toolchain.toml)."
            echo "cargo -> $(command -v cargo)"
            echo "rustc -> $(command -v rustc)"
          '';
        };
      }
    );
}
