# ch1 里 `cfg` / `cfg_attr` / 入口相关属性速记

本条目针对 `tg-rcore-tutorial-ch1/src/main.rs` 里你提到的两个 `cfg_attr` 以及入口布局相关属性，给出“看懂即可用”的解释。

## 1) `cfg_attr` 是什么

`cfg_attr(condition, attr1, attr2, ...)` 的含义是：

- 如果 `condition` 成立，就应用后面的属性（`attr1/attr2/...`）
- 如果不成立，就当这些属性不存在

它常用于“同一份源码在不同平台/构建模式下有不同的 lint、feature、行为”。

## 2) 本仓库 ch1 的两个 `cfg_attr`

在 ch1 顶部：

- `#![cfg_attr(target_arch = "riscv64", deny(warnings, missing_docs))]`
  - 含义：在真正的 RISC-V64 目标下，把 warning 当错误，并要求文档注释齐全。
  - 目的：让教学内核在真实目标上更“严格”。

- `#![cfg_attr(not(target_arch = "riscv64"), allow(dead_code))]`
  - 含义：在非 RISC-V64（比如你本机 x86_64）构建时允许死代码。
  - 目的：配合 `stub` 模块，让 `cargo publish --dry-run` 等在主机上也能通过编译。

## 3) 为什么 `_start` 需要 `link_section`，而 `rust_main/panic` 不需要

- `_start` 是早期启动入口：
  - 需要是一个“裸函数”（`#[unsafe(naked)]`），在没有栈时先把 `sp` 设好。
  - 放在 `.text.entry` 是为了配合链接脚本把它放到 S-mode 代码区开头（便于固定布局、符合文档描述，也便于调试）。

- `rust_main` 与 `panic_handler`：
  - 它们是被 `_start` 用符号跳转调用的普通函数。
  - 不需要固定在某个特定地址段，只要能被链接到 `.text` 即可。

## 4) `_m_start` 是什么

- `_m_start` 在 `tg-rcore-tutorial-sbi/src/m_entry.asm` 中定义，位于 `.text.m_entry`。
- 它作为整个 ELF 的入口（链接脚本 `ENTRY(_m_start)`），在 `-bios none` 场景下最先以 M-mode 执行。
- 随后它会把 `mepc` 设置为 S-mode 的 `_start`，并 `mret` 切到 S-mode。
