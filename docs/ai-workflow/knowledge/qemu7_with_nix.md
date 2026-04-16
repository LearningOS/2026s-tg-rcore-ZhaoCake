# qemu7 在 Nix 中的获取与固定（含本项目可用方案）

## 1. 结论

- 可以从 Nix 拉取 qemu7（即使当前 unstable 已经是 qemu10）。
- 最稳妥做法是 pin 到历史 nixpkgs commit。
- 本项目目前采用的可用版本是 qemu 7.2.1，对应：
	- `github:NixOS/nixpkgs/986a0a1a1905b3227b28db009619321105c62464#qemu`

## 2. 只用网页查版本历史（无需本地克隆）

### 方法 A：nixhub（最快）

1. 打开 https://www.nixhub.io/packages/qemu
2. 查找 `7.0.0` 或 `7.2.1` 出现的记录
3. 记下对应 branch 或 revision

### 方法 B：GitHub 历史（最精确）

1. 打开 nixpkgs 中 qemu 包目录：
	 - `pkgs/applications/virtualization/qemu/`
2. 进入 `default.nix`（或相关定义文件）
3. 点 History，搜索 `version = "7.*"`
4. 进入对应提交并复制 commit hash

### 方法 C：NixOS Search（按 release 过滤）

1. 打开 https://search.nixos.org/packages
2. 切换 `unstable` / `23.11` / `22.11` / `22.05` 等 release
3. 查看每个 release 的 qemu 版本

## 3. 在命令行快速验证

```bash
nix shell github:NixOS/nixpkgs/986a0a1a1905b3227b28db009619321105c62464#qemu -c qemu-system-riscv64 --version
```

如果你只要 qemu 7.x，通常 pin 到 release 就够；
如果你要精确版本（如 7.0.0），建议 pin 到 commit。

## 4. 与本仓库环境要求对齐

根据仓库 README 和章节工具链文件，本项目环境关键点是：

- Rust toolchain：`stable`
- Rust 组件：`rust-src`、`llvm-tools-preview`
- Rust 目标：`riscv64gc-unknown-none-elf`
- QEMU：`qemu-system-riscv64`（>=7.0）

所以用 qemu 7.2.1 是兼容本仓库要求的。

## 5. 推荐做法（本仓库）

- 用 `flake.nix` 固定 qemu 7.2.1 的 nixpkgs commit。
- Rust 继续交给 rustup（与章节 `rust-toolchain.toml` 对齐，成本最低）。
- 进入目录后通过 `.envrc` 自动加载 flake 环境。
