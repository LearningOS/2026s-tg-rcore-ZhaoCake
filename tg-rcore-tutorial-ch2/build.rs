use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::PathBuf, process::Command};

// 用户程序统一编译到该裸机目标，供内核打包和运行。
const TARGET_ARCH: &str = "riscv64gc-unknown-none-elf";

// 以下三项均从 .cargo/config.toml [env] 节读取，不再硬编码：
//   TG_USER_CRATE     — crates.io 包名
//   TG_USER_LOCAL_DIR — 本地缓存目录名
//   TG_USER_VERSION   — 版本号

#[derive(Deserialize, Default)]
struct Cases {
    // 用户程序装载基址。0 表示不拷贝到固定地址，直接使用链接进镜像的位置。
    base: Option<u64>,
    // 每个应用装载地址步长（当 base != 0 时生效）。
    step: Option<u64>,
    // 该章节要打包的用户程序二进制名列表。
    cases: Option<Vec<String>>,
}

/// build.rs 总入口。
///
/// 相比 ch1 的“仅写链接脚本”，这里扩展为三件事：
/// 1) 写内核链接脚本；
/// 2) 构建并收集用户程序；
/// 3) 生成 app.asm，并通过 APP_ASM 环境变量交给内核 `global_asm!` 引入。
fn main() {
    // 声明构建依赖，保证输入变化时重新执行 build.rs。
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LOG");
    println!("cargo:rerun-if-env-changed=TG_USER_DIR");
    println!("cargo:rerun-if-env-changed=TG_USER_CRATE");
    println!("cargo:rerun-if-env-changed=TG_USER_LOCAL_DIR");
    println!("cargo:rerun-if-env-changed=TG_USER_VERSION");

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // 只在 RISC-V64 架构进行真正打包。
    // 这样在主机平台做 metadata/publish 检查时不会误触发裸机构建流程。
    if target_arch == "riscv64" {
        // 第一步：链接脚本。
        write_linker();
        // 第二步：生成 APP_ASM。
        // 某些场景（如发布检查）可跳过用户程序构建，改用空占位 APP_ASM。
        if should_skip_build_apps() {
            write_dummy_app_asm();
        } else {
            build_apps();
        }
    }
}

/// 是否跳过用户程序构建。
///
/// 典型用途：
/// - 显式设置 TG_SKIP_USER_APPS；
/// - Cargo 在 target/package 下做发布打包检查（避免网络/外部命令依赖）。
fn should_skip_build_apps() -> bool {
    if env::var_os("TG_SKIP_USER_APPS").is_some() {
        return true;
    }

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let manifest_dir = manifest_dir.to_string_lossy();
    manifest_dir.contains("/target/package/") || manifest_dir.contains("\\target\\package\\")
}

/// 写入内核链接脚本并告知 rustc 使用它。
///
/// ch1 与 ch2 都要做这一步；差异在于 ch2 后续还要继续打包用户应用。
fn write_linker() {
    let ld = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("linker.ld");
    fs::write(&ld, tg_linker::NOBIOS_SCRIPT).unwrap_or_else(|err| {
        panic!("failed to write linker script to {}: {}", ld.display(), err)
    });
    println!("cargo:rustc-link-arg=-T{}", ld.display());
}

/// 构建并打包本章用户程序，生成 app.asm。
///
/// 流程：
/// 1) 定位 tg-user 源码；
/// 2) 读取 cases.toml 中 ch2 对应的程序列表；
/// 3) 逐个编译用户程序；
/// 4) 按 base/step 规则决定是否转 bin；
/// 5) 写 app.asm，并导出 APP_ASM 给内核编译单元使用。
fn build_apps() {
    let tg_user_root = ensure_tg_user();
    let cases_path = tg_user_root.join("cases.toml");
    // 一旦用户程序源码或清单变化，重新执行打包。
    println!("cargo:rerun-if-changed={}", cases_path.display());
    println!("cargo:rerun-if-changed={}", tg_user_root.join("Cargo.toml").display());
    println!("cargo:rerun-if-changed={}", tg_user_root.join("src").display());

    let cfg = fs::read_to_string(&cases_path).unwrap_or_else(|err| {
        panic!("failed to read cases.toml from {}: {}", cases_path.display(), err)
    });
    let mut cases_map: HashMap<String, Cases> = toml::from_str(&cfg).unwrap_or_else(|err| {
        panic!("failed to parse cases.toml: {err}")
    });

    let cases = cases_map.remove("ch2").unwrap_or_default();
    let base = cases.base.unwrap_or(0);
    let step = cases.step.unwrap_or(0);
    let names = cases.cases.unwrap_or_default();

    if names.is_empty() {
        panic!("no user cases found for ch2 in {}", cases_path.display());
    }

    let target_dir = tg_user_root.join("target").join(TARGET_ARCH).join("debug");
    let mut bins: Vec<PathBuf> = Vec::with_capacity(names.len());

    for (i, name) in names.iter().enumerate() {
        // base/step 由 cases.toml 控制。base==0 表示不指定固定装载地址。
        let base_address = base + i as u64 * step;
        build_user_app(&tg_user_root, name, base_address);
        let elf = target_dir.join(name);
        // 当需要固定地址装载时，转成纯二进制，便于内核拷贝到指定地址执行。
        let app_path = if base_address != 0 {
            objcopy_to_bin(&elf)
        } else {
            // 不固定地址时可直接用 ELF 切片。
            elf
        };
        bins.push(app_path);
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let app_asm = out_dir.join("app.asm");
    write_app_asm(&app_asm, base, step, &bins);
    // 提供给 src/main.rs: global_asm!(include_str!(env!("APP_ASM")))
    // 使 app.asm 在编译期并入内核镜像。
    println!("cargo:rustc-env=APP_ASM={}", app_asm.display());
}

/// 构建单个用户程序。
///
/// 注意：这里是“子 cargo build”，目标是 tg-user crate 内的指定 bin。
fn build_user_app(tg_user_root: &PathBuf, name: &str, base_address: u64) {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "build",
        "--manifest-path",
        tg_user_root.join("Cargo.toml").to_string_lossy().as_ref(),
        "--bin",
        name,
        "--target",
        TARGET_ARCH,
    ]);

    if base_address != 0 {
        // 传给用户程序构建过程，用于控制其链接基址。
        cmd.env("BASE_ADDRESS", base_address.to_string());
    }

    let status = cmd.status().expect("failed to execute cargo build for user app");
    if !status.success() {
        panic!("failed to build user app {name}");
    }
}

/// ELF -> bin。
///
/// 固定地址装载时，内核只需要纯指令/数据字节流，因此去符号并转 binary。
fn objcopy_to_bin(elf: &PathBuf) -> PathBuf {
    let bin = elf.with_extension("bin");
    let status = Command::new("rust-objcopy")
        .args([
            elf.to_string_lossy().as_ref(),
            "--strip-all",
            "-O",
            "binary",
            bin.to_string_lossy().as_ref(),
        ])
        .status()
        .expect("failed to execute rust-objcopy");
    if !status.success() {
        panic!("rust-objcopy failed for {}", elf.display());
    }
    bin
}

/// 写入 app.asm。
///
/// 该汇编文件定义符号 `apps`，其布局约定如下：
/// - [base, step, count]
/// - app_0_start..app_n_end 的地址表
/// - 每个应用的 `.incbin` 数据块
///
/// 运行期 `tg_linker::AppMeta::locate()` 会按同样布局解析它。
fn write_app_asm(path: &PathBuf, base: u64, step: u64, bins: &[PathBuf]) {
    use std::io::Write;
    let mut asm = fs::File::create(path)
        .unwrap_or_else(|err| panic!("failed to create {}: {}", path.display(), err));

    writeln!(
        asm,
        "\
.global apps
.section .data
.align 3
apps:
    .quad {base:#x}
    .quad {step:#x}
    .quad {}",
        bins.len(),
    )
    .unwrap();

    for i in 0..bins.len() {
        writeln!(asm, "    .quad app_{i}_start").unwrap();
    }

    writeln!(asm, "    .quad app_{}_end", bins.len() - 1).unwrap();

    for (i, path) in bins.iter().enumerate() {
        writeln!(
            asm,
            "\
app_{i}_start:
    # 把用户程序原始字节直接嵌入到内核镜像
    .incbin {path:?}
app_{i}_end:",
        )
        .unwrap();
    }
}

/// 生成空占位 app.asm。
///
/// 用于跳过用户应用构建时保持内核可编译（apps 元数据存在但 count=0）。
fn write_dummy_app_asm() {
    use std::io::Write;

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let app_asm = out_dir.join("app.asm");
    let mut asm = fs::File::create(&app_asm)
        .unwrap_or_else(|err| panic!("failed to create {}: {}", app_asm.display(), err));

    writeln!(
        asm,
        "\
.global apps
.section .data
.align 3
apps:
    .quad 0
    .quad 0
    .quad 0
    .quad 0"
    )
    .unwrap();

    println!("cargo:rustc-env=APP_ASM={}", app_asm.display());
}

/// 确保 tg-user 源码目录可用。
///
/// 优先级：
/// 1) 使用 TG_USER_DIR；
/// 2) 使用 .cargo/config.toml 的 crate/version 自动拉取；
/// 3) 拉取后修补 [workspace]，避免与父 workspace 冲突。
fn ensure_tg_user() -> PathBuf {
    // 优先使用 TG_USER_DIR 显式指定的目录
    if let Ok(dir) = env::var("TG_USER_DIR") {
        let path = PathBuf::from(dir);
        if path.join("Cargo.toml").exists() {
            return path;
        }
    }

    // 从 .cargo/config.toml [env] 读取三个配置项
    let crate_name = env::var("TG_USER_CRATE")
        .expect("TG_USER_CRATE not set; add it to .cargo/config.toml [env]");
    let local_dir_name = env::var("TG_USER_LOCAL_DIR")
        .expect("TG_USER_LOCAL_DIR not set; add it to .cargo/config.toml [env]");
    let version = env::var("TG_USER_VERSION")
        .expect("TG_USER_VERSION not set; add it to .cargo/config.toml [env]");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let tg_user_dir = manifest_dir.join(&local_dir_name);

    // 本地缓存目录已存在则直接使用
    if tg_user_dir.join("Cargo.toml").exists() {
        ensure_workspace_table(&tg_user_dir);
        return tg_user_dir;
    }

    // 从 crates.io 克隆指定包
    let crate_spec = format!("{crate_name}@{version}");
    let status = Command::new("cargo")
        .args([
            "clone",
            crate_spec.as_str(),
            "--",
            tg_user_dir.to_string_lossy().as_ref(),
        ])
        .status()
        .unwrap_or_else(|e| panic!("failed to execute cargo clone {crate_spec}: {e}"));

    if !status.success() {
        panic!(
            "failed to clone {crate_spec} into {}; ensure cargo-clone is installed or set TG_USER_DIR",
            tg_user_dir.display()
        );
    }

    if !tg_user_dir.join("Cargo.toml").exists() {
        panic!(
            "{crate_spec} clone did not produce a valid crate at {}",
            tg_user_dir.display()
        );
    }

    // 克隆后补加 [workspace]，防止父 workspace 将其识别为非成员而报错
    ensure_workspace_table(&tg_user_dir);

    tg_user_dir
}

/// 若 Cargo.toml 末尾尚无 [workspace] 表，则追加一个空的，
/// 使该 crate 成为独立 workspace 根，避免父 workspace 冲突。
fn ensure_workspace_table(dir: &PathBuf) {
    let cargo_toml = dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).unwrap_or_default();
    if !content.contains("[workspace]") {
        fs::write(&cargo_toml, format!("{}\n[workspace]\n", content))
            .unwrap_or_else(|err| panic!("failed to patch Cargo.toml in {}: {}", dir.display(), err));
    }
}
