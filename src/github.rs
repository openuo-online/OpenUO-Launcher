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

pub fn fetch_latest_release(url: &str) -> Result<GithubRelease> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Another-OpenUO-Launcher")
        .timeout(Duration::from_secs(8))
        .build()?;
    let resp = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()?
        .error_for_status()?
        .json::<GithubRelease>()?;
    Ok(resp)
}

pub fn download_and_unpack_open_uo_with_progress<F: Fn(DownloadEvent) + Send + 'static>(
    progress: F,
) -> Result<String> {
    let progress_cb = |evt: DownloadEvent| {
        progress(evt);
    };

    let release = fetch_latest_release(OPEN_UO_RELEASE_URL)?;
    
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

    let version = release.name.clone();
    write_open_uo_version(&version, &target_dir)?;
    Ok(version)
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
            let res = fetch_latest_release(OPEN_UO_RELEASE_URL)
                .map(|r| r.name)
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(UpdateEvent::OpenUO(res));
        }
        if launcher {
            let res = fetch_latest_release(LAUNCHER_RELEASE_URL)
                .map(|r| r.name)
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(UpdateEvent::Launcher(res));
        }
        let _ = tx.send(UpdateEvent::Done);
    });
    rx
}
