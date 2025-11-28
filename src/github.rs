use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use crate::config::open_uo_dir;

const OPEN_UO_RELEASE_URL: &str =
    "https://api.github.com/repos/openuo-online/OpenUO/releases/latest";
const LAUNCHER_RELEASE_URL: &str =
    "https://api.github.com/repos/openuo-online/Another-OpenUO-Launcher/releases/latest";
const OPEN_UO_VERSION_FILE: &str = ".open_uo_version";

// 自定义更新源配置文件
const UPDATE_SOURCE_CONFIG: &str = "update_source.json";

/// 自定义更新源配置
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSourceConfig {
    /// OpenUO 更新信息 URL（可以是 GitHub API 或自定义 JSON）
    pub openuo_url: Option<String>,
    /// Launcher 更新信息 URL
    pub launcher_url: Option<String>,
    /// 是否使用 GitHub API 格式（false 则使用简化格式）
    #[serde(default = "default_true")]
    pub use_github_format: bool,
}

fn default_true() -> bool {
    true
}

/// 简化的更新信息格式（用于自定义 CDN）
#[derive(Debug, Clone, Deserialize)]
pub struct SimpleRelease {
    /// 版本号/标签
    pub version: String,
    /// 下载 URL（可以是对象或字符串）
    pub download_url: DownloadUrls,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DownloadUrls {
    /// 单个 URL（自动根据平台选择）
    Single(String),
    /// 多平台 URL
    Multiple {
        #[serde(rename = "osx-arm64")]
        osx_arm64: Option<String>,
        #[serde(rename = "osx-x64")]
        osx_x64: Option<String>,
        #[serde(rename = "linux-x64")]
        linux_x64: Option<String>,
        #[serde(rename = "win-x64")]
        win_x64: Option<String>,
    },
}

fn get_platform_asset_name() -> String {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "osx-arm64.zip".to_string();
    
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "osx-x64.zip".to_string();
    
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "linux-x64.zip".to_string();
    
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "win-x64.zip".to_string();
    
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    {
        panic!("不支持的平台");
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<GithubAsset>,
    pub body: Option<String>,
    pub published_at: Option<String>,
    pub target_commitish: Option<String>,
}

pub enum DownloadEvent {
    Progress { received: u64, total: u64 },
    Finished(Result<String, String>),
}

pub enum UpdateEvent {
    OpenUO(Result<String, String>),
    Launcher(Result<String, String>),
    Done,
}

/// 加载自定义更新源配置
fn load_update_source_config() -> Option<UpdateSourceConfig> {
    let config_path = crate::config::base_dir().join(UPDATE_SOURCE_CONFIG);
    if !config_path.exists() {
        return None;
    }
    
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str::<UpdateSourceConfig>(&content) {
                Ok(config) => {
                    tracing::info!("使用自定义更新源配置");
                    Some(config)
                }
                Err(e) => {
                    tracing::warn!("解析更新源配置失败: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!("读取更新源配置失败: {}", e);
            None
        }
    }
}

/// 获取 OpenUO 更新 URL
fn get_openuo_update_url() -> String {
    load_update_source_config()
        .and_then(|c| c.openuo_url)
        .unwrap_or_else(|| OPEN_UO_RELEASE_URL.to_string())
}

/// 获取 Launcher 更新 URL
fn get_launcher_update_url() -> String {
    load_update_source_config()
        .and_then(|c| c.launcher_url)
        .unwrap_or_else(|| LAUNCHER_RELEASE_URL.to_string())
}

/// 是否使用 GitHub API 格式
fn use_github_format() -> bool {
    load_update_source_config()
        .map(|c| c.use_github_format)
        .unwrap_or(true)
}

pub fn fetch_latest_release(url: &str) -> Result<GithubRelease> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Another-OpenUO-Launcher")
        .timeout(Duration::from_secs(8))
        .build()?;
    
    if use_github_format() {
        // GitHub API 格式
        let resp = client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .send()?
            .error_for_status()?
            .json::<GithubRelease>()?;
        Ok(resp)
    } else {
        // 简化格式，转换为 GithubRelease
        let resp = client
            .get(url)
            .send()?
            .error_for_status()?
            .json::<SimpleRelease>()?;
        
        // 转换为 GithubRelease 格式
        let platform_name = get_platform_asset_name();
        let download_url = match resp.download_url {
            DownloadUrls::Single(url) => url,
            DownloadUrls::Multiple { osx_arm64, osx_x64, linux_x64, win_x64 } => {
                #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
                let url = osx_arm64;
                
                #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
                let url = osx_x64;
                
                #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
                let url = linux_x64;
                
                #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
                let url = win_x64;
                
                url.context("当前平台没有可用的下载链接")?
            }
        };
        
        Ok(GithubRelease {
            tag_name: resp.version.clone(),
            name: resp.version,
            assets: vec![GithubAsset {
                name: platform_name,
                browser_download_url: download_url,
                size: 0,
            }],
            body: None,
            published_at: None,
            target_commitish: None,
        })
    }
}

pub fn download_and_unpack_open_uo_with_progress<F: Fn(DownloadEvent) + Send + 'static>(
    progress: F,
) -> Result<String> {
    let progress_cb = |evt: DownloadEvent| {
        progress(evt);
    };

    let url = get_openuo_update_url();
    let release = fetch_latest_release(&url)?;
    
    // 根据当前平台选择正确的资产
    let platform_name = get_platform_asset_name();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == platform_name)
        .cloned()
        .context(format!("未找到平台 {} 的资产", platform_name))?;

    let tmp = std::env::temp_dir().join(&asset.name);
    download_asset(&asset.browser_download_url, &tmp, |received, total| {
        progress_cb(DownloadEvent::Progress { received, total });
    })?;

    let target_dir = open_uo_dir();
    fs::create_dir_all(&target_dir)?;
    extract_zip(&tmp, &target_dir)?;
    fs::remove_file(&tmp).ok();

    // 使用发布时间作为版本标识
    let version = get_version_string(&release);
    write_open_uo_version(&version, &target_dir)?;
    Ok(version)
}

pub fn download_launcher_update<F: Fn(DownloadEvent) + Send + 'static>(
    progress: F,
) -> Result<String> {
    let progress_cb = |evt: DownloadEvent| {
        progress(evt);
    };

    let url = get_launcher_update_url();
    let release = fetch_latest_release(&url)?;
    
    // 根据当前平台选择正确的可执行文件
    let launcher_name = get_launcher_asset_name();
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == launcher_name)
        .cloned()
        .context(format!("未找到平台 {} 的 Launcher", launcher_name))?;

    // 下载到临时文件
    let tmp = std::env::temp_dir().join(&asset.name);
    download_asset(&asset.browser_download_url, &tmp, |received, total| {
        progress_cb(DownloadEvent::Progress { received, total });
    })?;

    // 设置执行权限（Unix 系统）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&tmp, perms)?;
    }
    
    // 获取当前可执行文件路径（在替换前）
    let current_exe = std::env::current_exe()?;
    
    // 使用 self_replace 替换当前可执行文件
    // 这个库会自动处理跨平台的替换逻辑
    self_replace::self_replace(&tmp)?;
    
    // 删除临时文件
    fs::remove_file(&tmp).ok();
    
    let version = get_version_string(&release);
    
    // 启动新版本程序
    #[cfg(target_os = "macos")]
    {
        // macOS 使用 open 命令
        std::process::Command::new("open")
            .arg(&current_exe)
            .spawn()
            .ok();
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows 下使用 cmd /c start 来启动
        // 使用 start "" "path" 格式，空字符串是窗口标题
        let exe_str = current_exe.to_string_lossy().to_string();
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", &exe_str])
            .spawn()
            .ok();
    }
    
    #[cfg(target_os = "linux")]
    {
        // Linux 直接启动
        std::process::Command::new(&current_exe)
            .spawn()
            .ok();
    }
    
    // 返回特殊标记，告诉 UI 需要退出程序
    Ok(format!("UPDATE_AND_RESTART:{}", version))
}

fn get_launcher_asset_name() -> String {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return "OpenUO-Launcher-macos-arm64".to_string();
    
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return "OpenUO-Launcher-macos-x64".to_string();
    
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return "OpenUO-Launcher-linux-x64".to_string();
    
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return "OpenUO-Launcher-windows-x64.exe".to_string();
    
    #[cfg(not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64")
    )))]
    {
        panic!("不支持的平台");
    }
}

fn download_asset(url: &str, dest: &PathBuf, progress: impl Fn(u64, u64)) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Another-OpenUO-Launcher")
        .timeout(Duration::from_secs(8))
        .build()?;
    let mut resp = client.get(url).send()?.error_for_status()?;
    let mut file = fs::File::create(dest)?;
    let total = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    let mut received = 0u64;
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let n = resp.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        received += n as u64;
        progress(received, total);
    }
    Ok(())
}

fn extract_zip(zip_path: &PathBuf, target_dir: &PathBuf) -> Result<()> {
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let mut outpath = target_dir.clone();
        outpath.push(file.mangled_name());

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }
    Ok(())
}

fn write_open_uo_version(tag: &str, dir: &PathBuf) -> Result<()> {
    let path = dir.join(OPEN_UO_VERSION_FILE);
    fs::write(path, tag)?;
    Ok(())
}

pub fn read_open_uo_version_file() -> Option<String> {
    let path = open_uo_dir().join(OPEN_UO_VERSION_FILE);
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

pub fn detect_open_uo_version() -> Option<String> {
    let exe = crate::config::open_uo_binary_path();
    if !exe.exists() {
        return None;
    }
    if let Some(ver) = read_open_uo_version_file() {
        return Some(ver);
    }
    Some("已安装 (版本未知)".to_string())
}

pub fn trigger_update_check_impl(open_uo: bool, launcher: bool) -> mpsc::Receiver<UpdateEvent> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        if open_uo {
            let url = get_openuo_update_url();
            let res = fetch_latest_release(&url)
                .map(|r| get_version_string(&r))
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(UpdateEvent::OpenUO(res));
        }
        if launcher {
            let url = get_launcher_update_url();
            let res = fetch_latest_release(&url)
                .map(|r| get_version_string(&r))
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(UpdateEvent::Launcher(res));
        }
        let _ = tx.send(UpdateEvent::Done);
    });
    rx
}

// 从 release 中提取版本字符串
fn get_version_string(release: &GithubRelease) -> String {
    // 直接使用 release 的 name 字段作为版本号
    release.name.clone()
}
