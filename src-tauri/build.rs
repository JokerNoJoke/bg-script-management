fn main() {
    // tauri-build 只为正式二进制嵌入 Common-Controls v6 清单；启用 tauri 的
    // `test` feature 后测试二进制会链接 comctl32 v6，缺少清单导致
    // STATUS_ENTRYPOINT_NOT_FOUND 加载失败，因此对测试目标单独嵌入清单。
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let manifest = format!(
            "{}/windows-app-manifest.xml",
            std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default()
        );
        println!("cargo:rerun-if-changed={manifest}");
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{manifest}");
    }
    tauri_build::build()
}
