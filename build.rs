fn main() {
    // 仅在启用 gui feature 时运行 Tauri 构建脚本
    #[cfg(feature = "gui")]
    {
        tauri_build::build();
    }
}
