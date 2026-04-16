# cargo run 与 bare-metal 运行器机制

## 1. 问题定义

在 bare-metal 项目中，为什么只执行 `cargo run` 就会出现：

- `Running qemu-system-riscv64 ...`
- 并实际在 QEMU 中启动内核镜像？

## 2. 关键结论

`cargo run` 并不总是“直接运行本机二进制”。
当目标是交叉编译 target 时，Cargo 会读取 `.cargo/config.toml` 中的 `runner` 配置，把编译产物交给 runner 执行。

## 3. 在本仓库 ch1 的对应关系

- 默认目标：`riscv64gc-unknown-none-elf`
- runner：`qemu-system-riscv64 -machine virt -nographic -bios none -kernel`

因此在 ch1 中：

- `test.sh` 只写 `cargo run` 是正常的。
- 具体 qemu 命令来自 `.cargo/config.toml`，不是 `test.sh` 手写。

## 4. build.rs 在这里做什么

`build.rs` 的职责是构建期处理（如链接脚本、链接参数、内存布局），
不是运行期选择器。是否调用 qemu 由 Cargo 的 runner 决定。

## 5. 快速排查模板

当你想知道 `cargo run` 究竟执行了什么，按顺序检查：

1. `.cargo/config.toml`
- `build.target`
- `target.<triple>.runner`

2. `Cargo.toml`
- 依赖、特性、profile（panic、lto 等）

3. `build.rs`
- 链接脚本、编译参数注入

## 6. 常见误区

- 误区 1：`cargo run` 一定是本机运行。
  - 实际：对于 cross target，常由 runner 接管。

- 误区 2：看到 QEMU 就不算 bare-metal。
  - 实际：QEMU 只是硬件模拟器，目标程序仍是裸机程序。

- 误区 3：`build.rs` 决定运行命令。
  - 实际：`build.rs` 管“怎么编译/链接”，`runner` 管“怎么运行”。
