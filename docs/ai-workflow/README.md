# AI 协作学习执行指南（tg-rcore-tutorial）

本文给出一套可直接执行的学习路线，帮助你在本仓库中完成课程任务，并沉淀与 AI 协作的过程文档。

## 1. 你应该按什么步骤做

### Step 0：准备环境并验证工具

在仓库根目录先确认基础依赖：

```bash
rustup target add riscv64gc-unknown-none-elf
rustup component add rust-src llvm-tools-preview
qemu-system-riscv64 --version
```

可选工具（建议安装）：

```bash
cargo install cargo-binutils cargo-clone
cargo install --path tg-rcore-tutorial-checker
```

### Step 1：先跑通章节内核，建立基线

建议顺序：ch1 -> ch2 -> ch3 -> ch4 -> ch5 -> ch6 -> ch7 -> ch8。

每章至少做一次：

```bash
cd tg-rcore-tutorial-chN
cargo build
cargo run
./test.sh base
```

目标：确保你对每章默认行为、日志风格、测试入口有统一认知。

### Step 2：完成 5 个基础实验（重点）

必做章节：ch3 / ch4 / ch5 / ch6 / ch8。

推荐顺序与每章主任务：

1. ch3
- 新增 `trace` 系统调用（ID 410）
- 支持读/写当前任务地址与 syscall 次数统计

2. ch4
- 重写 `trace`（引入地址空间后的权限检查）
- 实现 `mmap`（ID 222）与 `munmap`（ID 215）

3. ch5
- 迁移 `mmap/munmap` 到新进程结构
- 实现 `spawn`（ID 400）
- 实现 stride 调度与 `set_priority`（ID 140）

4. ch6
- 实现 `linkat`（ID 37）、`unlinkat`（ID 35）、`fstat`（ID 80）
- 必要时拉取并本地修改 `tg-rcore-tutorial-easy-fs`

5. ch8
- 实现死锁检测
- 实现 `enable_deadlock_detect`（ID 469）
- 死锁场景下 `mutex_lock`/`semaphore_down` 返回 `-0xDEAD`

每章完成后建议执行：

```bash
cargo run --features exercise
./test.sh exercise
```

并用 checker 交叉验证输出（按需）：

```bash
cargo run --features exercise 2>&1 | tg-rcore-tutorial-checker --ch N --exercise
```

### Step 3：做前向兼容回归

根据章节要求做回归：

- ch5：需通过前一章测例（除 ch3_trace/ch4_trace）
- ch6：需通过前一章全部测例
- ch8：需通过 ch8 全部测例 + 其他章节基础测例

目标：避免“新功能通过但旧功能回归失败”。

### Step 4：选择拓展线（至少选 1 条，建议按时间）

A. 改进教程线（推荐）
- 对仓库做扩展/裁剪/重构/组件重组
- 形成你自己的个性化教程分支

B. 扩展实验线（游戏 + 内核）
- 在 ch1~ch8 对应实现游戏应用与内核支持
- 示例包括 snake/tetris/pingpong/breakout/pacman/doom

C. 挑战线（docs/challenges.md）
- 支持内核态响应中断（ch2-8）
- 支持 SMP 多处理器并行处理（ch2-8）

### Step 5：完成课程交付

你的最终交付至少应包含：

1. 实现结果
- 代码与测试通过记录

2. 与 AI 合作过程
- 关键提示词
- 遇到的问题/bug
- 定位与修复过程
- 最终验证证据

3. 学习效果评估
- 自评能力变化
- 与本校现有实验教程的定量/定性对比

## 2. 你需要完成哪些任务（清单版）

### 必做（核心）

- 跑通 ch1~ch8 默认流程（至少 base）
- 完成 ch3/ch4/ch5/ch6/ch8 的 exercise
- 每章保留可复现实验记录（命令、结果、问题、修复）
- 输出一份含 AI 协作过程的总结报告

### 选做（加分）

- 改进教程（结构重构、组件化改造、学习路径优化）
- 扩展游戏应用并驱动内核扩展
- 完成挑战任务：内核态中断 + SMP

## 3. AI 交互文档应该放在哪里

建议放在 `docs/` 下新建专门目录，这是最清晰、最易评阅、最便于长期复盘的方案。

本仓库建议使用：

- `docs/ai-workflow/README.md`：总路线与任务管理
- `docs/ai-workflow/ai-collaboration-template.md`：统一记录模板
- `docs/ai-workflow/knowledge/`：沉淀可复用的知识条目（环境、工具链、排障）
- `docs/ai-workflow/logs/`：按日期保存每次 AI 协作会话

建议分层规则：

- `logs/` 记录过程（你做了什么、遇到了什么）
- `knowledge/` 记录结论（以后可直接复用的做法）

这样做的好处：

- 与课程文档同级，提交时结构清晰
- 可直接关联章节和提交记录
- 便于你后续写课程总结/答辩材料

## 4. 推荐执行节奏（两周样例）

1. 第 1-3 天：环境 + ch1/ch2/ch3
2. 第 4-6 天：ch4/ch5
3. 第 7-9 天：ch6
4. 第 10-12 天：ch8 + 回归
5. 第 13-14 天：拓展线 + 报告整理

如果时间紧，优先级始终是：

1. 5 个基础实验全部通过
2. 回归通过
3. 文档完整
4. 再做拓展与挑战
