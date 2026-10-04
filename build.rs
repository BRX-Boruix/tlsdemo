//! 用户态程序链接脚本注入。
//!
//! 通过 `CARGO_MANIFEST_DIR` 得到绝对路径，把 `linker.ld` 传给链接器，
//! 将 `.text` 等段定位到用户态地址（0x400000 起），`ENTRY(_start)`。
//!
//! **3P4-6 起本程序保持 rust-lld 默认的 PIE（ET_DYN）**：内核加载器已支持 ET_DYN
//! （装载基址偏移 + 段内页偏移映射 LM1），故不再需要 `-no-pie` 这个绕过。保留它会让
//! 本程序成为「内核不支持 PIE」的活化石，也让 PIE 路径无人验证——本程序正是那条路径
//! 的端到端载体（同时覆盖 ET_DYN 下 PT_TLS 的偏移）。
//!
//! 注意：缺 `-T linker.ld` 注入会把段链接到 vaddr 0x0（与内核用户半区冲突），
//! 加载器无法装载——链接脚本注入仍然是必需的。

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "none" {
        let dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        println!("cargo:rustc-link-arg=-T{}/linker.ld", dir);
        println!("cargo:rustc-link-arg=-no-pie");
    }
}
