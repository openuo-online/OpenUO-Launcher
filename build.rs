fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        
        // 设置图标（如果存在）
        if std::path::Path::new("assets/icon.ico").exists() {
            res.set_icon("assets/icon.ico");
        }
        
        // 设置应用程序信息
        res.set("ProductName", "OpenUO Launcher");
        res.set("FileDescription", "Another OpenUO Launcher");
        res.set("CompanyName", "OpenUO Contributors");
        res.set("LegalCopyright", "BSD-2-Clause License");
        
        // 编译资源
        if let Err(e) = res.compile() {
            eprintln!("Warning: Failed to compile Windows resources: {}", e);
        }
    }
}
